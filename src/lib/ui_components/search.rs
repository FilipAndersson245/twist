use std::{error::Error, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Text},
    Frame,
};

#[derive(Debug, Default, Clone)]
pub struct Search {}

impl Search {
    pub fn draw(
        &mut self,
        painter: &mut Frame<CrosstermBackend<Stdout>>,
        chunk: Rect,
        query: &str,
    ) -> Result<(), Box<dyn Error>> {
        let block = Block::default()
            .title_style(Style::default().fg(Color::Red))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black))
            .title("SearchBox");

        let paragraph = [Text::raw(query)];
        let paragraph = Paragraph::new(paragraph.iter())
            .block(block)
            .alignment(Alignment::Center)
            .wrap(true);

        painter.render_widget(paragraph, chunk);

        Ok(())
    }
}
