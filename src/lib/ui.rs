use crate::{
    api::{fetch_all_animes, fetch_anime, fetch_video},
    types::{Anime, Animes, DownloadInfo, Episode},
    ui_components::{
        anime::AnimeList, episodes::EpisodeList, notifications::Notification, progress::Progress,
        search::Search,
    },
};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::{
    collections::VecDeque,
    error::Error,
    io::{stdout, Stdout, Write},
    time::Duration,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Text,
    Terminal,
};

#[derive(Debug)]
pub struct App {
    state: State,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    ui: Ui,
}

#[derive(Debug, Clone)]
pub enum SelectMode {
    Anime,
    Episode,
}

impl Default for SelectMode {
    fn default() -> Self {
        SelectMode::Anime
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub select_mode: SelectMode,
    pub animes: Animes,
    pub selected_anime: Anime,
    pub query: String,
    pub download_progress: Option<(u64, u64)>,
    pub download_queue: VecDeque<DownloadInfo>,
}

#[derive(Default, Debug)]
pub struct Ui {
    pub notification: Notification,
    pub search: Search,
    pub episodes: EpisodeList,
    pub anime: AnimeList,
    pub progress: Progress,
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyboardInput(KeyEvent),
    AnimeSelected(Anime),
    EpisodeSelected(Episode),
    Download(DownloadMessage),
    Notification(Text<'static>),
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    Progress(u64, u64),
    Finished,
    Starting,
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<Message>(50);
        Self {
            sender,
            receiver,
            state: Default::default(),
            ui: Default::default(),
        }
    }

    pub fn query(&self) -> Animes {
        let matcher = SkimMatcherV2::default();

        self.state
            .animes
            .iter()
            .filter(|anime| {
                if self.state.query.len() < 1 {
                    return true;
                }
                let title: &str = &anime.title;
                return match matcher.simple_match(
                    &title.clone(),
                    &self.state.query.clone(),
                    false,
                    true,
                ) {
                    Some((m, _)) if m > 10 => true,
                    _ => {
                        let alt_title: &str = &anime.alt_title.clone().unwrap_or_default();
                        return match matcher.simple_match(
                            alt_title,
                            &self.state.query.clone(),
                            false,
                            true,
                        ) {
                            Some((m2, _)) if m2 > 10 => true,
                            _ => false,
                        };
                    }
                };
            })
            .cloned()
            .collect()
    }

    pub async fn draw(
        &mut self,
        t: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        t.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            let (list_chunk, right_chunk) = (chunks[0], chunks[1]);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(3),
                        Constraint::Min(3),
                        Constraint::Percentage(95),
                    ]
                    .as_ref(),
                )
                .split(right_chunk);
            let (search_chunk, download_chunk, chunk) = (chunks[0], chunks[1], chunks[2]);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(chunk);
            let (episode_chunk, notification_chunk) = (chunks[0], chunks[1]);

            self.ui
                .search
                .draw(&mut f, search_chunk, &self.state.query)
                .unwrap();

            self.ui.anime.draw(&mut f, list_chunk).unwrap();

            self.ui.episodes.draw(&mut f, episode_chunk).unwrap();

            if let Some((a, b)) = self.state.download_progress {
                self.ui.progress.draw(&mut f, download_chunk, a, b).unwrap();
            }

            self.ui
                .notification
                .draw(&mut f, notification_chunk)
                .unwrap();
        })?;

        Ok(())
    }

    async fn on_message(&mut self, msg: Message) -> Result<(), Box<dyn Error>> {
        match msg {
            Message::KeyboardInput(msg) => {
                self.on_keyboard_message(msg).await?;
            }
            Message::AnimeSelected(anime) => {
                self.state.selected_anime = anime.clone();
                self.state.select_mode = SelectMode::Episode;

                let episodes = fetch_anime(&anime).await?;
                self.ui.episodes = EpisodeList::with_items(episodes);
                let text = Text::styled(
                    format!("{:?}", anime.clone()),
                    Style::new().fg(Color::LightBlue),
                );
                self.sender.send(Message::Notification(text)).await?;
            }
            Message::EpisodeSelected(episode) => {
                self.state.download_queue.push_back(DownloadInfo(
                    self.state.selected_anime.clone(),
                    episode.clone(),
                ));

                if self.state.download_queue.len() == 1 {
                    let DownloadInfo(anime, episode) =
                        self.state.download_queue.front().unwrap().clone();

                    let sender = self.sender.clone();
                    tokio::spawn(async move {
                        let _ = fetch_video(&episode, &anime, sender).await;
                    });
                }
            }
            Message::Download(msg) => {
                self.on_download_message(msg).await?;
            }
            Message::Notification(text) => {
                self.ui.notification.update(text);
            }
        };
        Ok(())
    }

    async fn on_download_message(&mut self, msg: DownloadMessage) -> Result<(), Box<dyn Error>> {
        //self.ui.notification.update(Text::raw(format!("{:?}", msg))); // Tmp, may improve later.
        match msg {
            DownloadMessage::Progress(progress, total) => {
                self.state.download_progress = Some((progress, total));
            }
            DownloadMessage::Finished => {
                let text = Text::styled("finished", Style::new().fg(Color::LightBlue));
                self.state.download_progress = None;
                self.state.download_queue.pop_front();
                self.sender.send(Message::Notification(text)).await?;

                if self.state.download_queue.len() > 0 {
                    let DownloadInfo(anime, episode) =
                        self.state.download_queue.pop_front().unwrap();
                    let sender = self.sender.clone();
                    tokio::spawn(async move {
                        let _ = fetch_video(&episode, &anime, sender).await;
                    });
                }
            }
            DownloadMessage::Starting => {
                let text = Text::styled("Starting", Style::new().fg(Color::LightBlue));
                self.sender.send(Message::Notification(text)).await?;
            }
        }
        Ok(())
    }

    async fn on_keyboard_message(&mut self, msg: KeyEvent) -> Result<(), Box<dyn Error>> {
        match msg.code {
            KeyCode::Backspace => {
                self.state.query.pop();
                let anime = self.query();
                self.ui.anime = AnimeList::with_items(anime);
            }
            KeyCode::Enter => match self.state.select_mode {
                SelectMode::Anime => {
                    if let Some(idx) = self.ui.anime.state.selected() {
                        let anime = self.ui.anime.items.get(idx).unwrap();
                        self.sender
                            .send(Message::AnimeSelected(anime.clone()))
                            .await?;
                    };
                }
                SelectMode::Episode => {
                    if let Some(idx) = self.ui.episodes.state.selected() {
                        let episode = self.ui.episodes.items.get(idx).unwrap();
                        self.sender
                            .send(Message::EpisodeSelected(episode.clone()))
                            .await?;
                    };
                }
            },
            KeyCode::Left => {}
            KeyCode::Right => {}
            KeyCode::Up => match self.state.select_mode {
                SelectMode::Anime => {
                    self.ui.anime.previous();
                }
                SelectMode::Episode => {
                    self.ui.episodes.previous();
                }
            },
            KeyCode::Down => match self.state.select_mode {
                SelectMode::Anime => {
                    self.ui.anime.next();
                }
                SelectMode::Episode => {
                    self.ui.episodes.next();
                }
            },
            KeyCode::Home => {}
            KeyCode::End => {}
            KeyCode::PageUp => {}
            KeyCode::PageDown => {}
            KeyCode::Tab => {}
            KeyCode::BackTab => {}
            KeyCode::Delete => {}
            KeyCode::Insert => {}
            KeyCode::F(_) => {}
            KeyCode::Char(c) => {
                self.state.query.push(c);
                let anime = self.query();
                self.ui.anime = AnimeList::with_items(anime);
            }
            KeyCode::Null => {}
            KeyCode::Esc => match self.state.select_mode {
                SelectMode::Anime => {
                    self.on_exit()?;
                }
                SelectMode::Episode => {
                    self.state.select_mode = SelectMode::Anime;
                }
            },
        };

        Ok(())
    }

    fn on_exit(&self) -> Result<(), Box<dyn Error>> {
        // Until better way found, just crash out.
        panic!()
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.state.animes = fetch_all_animes().await?;
        // Configure terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        // Setup event listener for keypresses.
        let mut key_sender = self.sender.clone();
        // Maybe move to own function.
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            loop {
                let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
                let mut event = reader.next().fuse();

                select! {
                    _ = delay => {},
                    maybe_event = event => {
                        match maybe_event.unwrap().unwrap() {
                            Event::Key(key) => { if let Err(_) =key_sender.send(Message::KeyboardInput(key)).await {
                                println!("receiver dropped, while program where running!");
                                return;
                            } },
                            _ => {}
                        }
                    }
                }
            }
        });

        self.ui.anime = AnimeList::with_items(self.state.animes.clone());

        self.draw(&mut terminal).await?;
        // Initilize eventloop.
        while let Some(msg) = self.receiver.recv().await {
            self.on_message(msg).await?;
            self.draw(&mut terminal).await?;
        }
        Ok(())
    }
}
