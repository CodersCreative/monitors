use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType},
};

pub mod history;
pub mod monitor;
pub mod processes;
pub mod stats;

pub fn get_block() -> Block<'static> {
    Block::bordered()
        .title_alignment(Alignment::Center)
        .title_style(Style::default().fg(Color::Red))
        .border_type(BorderType::Rounded)
}
