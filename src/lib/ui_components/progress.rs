use crate::pretty_bytes::convert;
use std::{error::Error, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Text},
    Frame,
};

#[derive(Debug, Default, Clone)]
pub struct Progress {}

impl Progress {
    pub fn draw(
        &mut self,
        painter: &mut Frame<CrosstermBackend<Stdout>>,
        chunk: Rect,
        download_bytes: u64,
        total_size: u64,
    ) -> Result<(), Box<dyn Error>> {
        let progress = download_bytes as f64 / total_size as f64;
        let progress = if progress > 1.0 { 1.0 } else { progress };

        let download_bytes = convert(download_bytes);
        let total_size = convert(total_size);

        let label = format!(
            "{:.2}% \t {} / {}",
            progress * 100.0,
            download_bytes,
            total_size
        );

        let block = Block::default()
            .title_style(Style::default().fg(Color::Red))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black))
            .title("Download:");

        let gauge = Gauge::default()
            .block(block)
            .style(
                Style::default()
                    .fg(Color::LightCyan)
                    .bg(Color::Black)
                    .modifier(Modifier::ITALIC | Modifier::BOLD),
            )
            .label(&label)
            .ratio(progress);

        painter.render_widget(gauge, chunk);

        Ok(())
    }
}
