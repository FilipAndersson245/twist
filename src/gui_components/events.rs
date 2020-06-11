//use std::sync::mpsc::{channel, Receiver};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::types::anime::{Anime, Episode};
use crossterm::event::{self, Event as CEvent};
use event::KeyEvent;

#[derive(Debug, Clone)]
pub enum Event {
    Input(KeyEvent),
    Tick,
}

#[derive(Debug, Clone)]
pub enum Event2 {
    Input(KeyEvent),
    Download(usize),
    Tick,
}

pub struct TwoWayChannel<S, R> {
    pub sender: Sender<S>,
    pub receiver: Receiver<R>,
}

impl<S, R> TwoWayChannel<S, R> {
    pub fn new(sender: Sender<S>, receiver: Receiver<R>) -> Self {
        Self { sender, receiver }
    }
}

pub fn create_ui_logic_channels<R, T>(
) -> Result<(TwoWayChannel<T, R>, TwoWayChannel<R, T>), Box<dyn Error>> {
    let (i_s, e_r): (Sender<T>, Receiver<T>) = channel(5); // Jobs in queue before blocking.
    let (e_s, i_r): (Sender<R>, Receiver<R>) = channel(5);

    let a = TwoWayChannel::new(i_s, i_r);
    let b = TwoWayChannel::new(e_s, e_r);
    Ok((a, b))
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    Input(KeyEvent),
    Anime(Anime),
    Episode(Episode),
    Tick,
}

#[derive(Debug, Clone)]
pub enum LogicEvent {
    DownloadStarted(String),
    DownloadFinished(String),
    DownloadProgress(usize),
}

pub async fn foo() {
    let (mut ui_channel, mut logic_channel) =
        create_ui_logic_channels::<UIEvent, LogicEvent>().unwrap();

    //  ui_channel.sender.send_timeout(value, timeout)

    tokio::spawn(async move {});

    ()
}

// pub async fn create_event_thread(ms: u64) -> Receiver<Event> {
//     let (mut sender, receiver) = channel(1);

//     let tick_rate = Duration::from_millis(ms);
//     tokio::spawn(async move {
//         let mut last_tick = Instant::now();

//         loop {
//             // Handle keyboard events.

//             if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
//                 if let CEvent::Key(key) = event::read().unwrap() {
//                     if let Err(_) = sender.send(Event::Input(key)).await {
//                         println!("Error dispatching keyboard event");
//                     }
//                 }
//             }
//             // Tick the time every tick_rate.
//             if last_tick.elapsed() >= tick_rate {
//                 if let Err(_) = sender.send(Event::Tick).await {
//                     println!("Error dispatching tick.")
//                 }
//                 last_tick = Instant::now();
//             }
//         }
//     });
//     receiver
// }
