use super::statefull_list::StatefulList;
use crate::types::Anime;

use std::{error::Error, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Frame,
};

pub type AnimeList = StatefulList<Anime>;

impl AnimeList {
    pub fn draw(
        &mut self,
        painter: &mut Frame<CrosstermBackend<Stdout>>,
        chunk: Rect,
    ) -> Result<(), Box<dyn Error>> {
        let style = Style::default();
        let items = self.items.iter().map(|i| Text::raw(i.title.clone()));

        let items = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search result"),
            )
            .style(style)
            .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
            .highlight_symbol(">");

        painter.render_stateful_widget(items, chunk, &mut self.state);

        Ok(())
    }
}
