use std::collections::VecDeque;
use std::{error::Error, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, Text},
    Frame,
};

#[derive(Debug, Clone)]
pub struct Notification {
    rows: VecDeque<Text<'static>>,
}

impl Default for Notification {
    fn default() -> Self {
        Notification::new()
    }
}

impl Notification {
    pub fn new() -> Self {
        let mut rows = VecDeque::with_capacity(10);
        for _ in 0..10 {
            rows.push_back(Text::raw(""))
        }

        Self { rows }
    }

    pub fn draw(
        &mut self,
        painter: &mut Frame<CrosstermBackend<Stdout>>,
        chunk: Rect,
    ) -> Result<(), Box<dyn Error>> {
        let block = Block::default()
            .title_style(Style::default().fg(Color::Red))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black))
            .title("Notifications");

        let paragraphs = List::new(self.rows.iter().cloned()).block(block);

        // let paragraphs = Paragraph::new(self.rows.iter())
        //     .block(block)
        //     .alignment(Alignment::Left)
        //     .wrap(false);

        painter.render_widget(paragraphs, chunk);

        Ok(())
    }

    pub fn update(&mut self, text: Text<'static>) {
        self.rows.pop_front();
        self.rows.push_back(text);
    }
}
