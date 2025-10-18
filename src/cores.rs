use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Gauge},
    Frame,
};

use crate::stats::get_block;

const Y: usize = 12;
const X: usize = 5;

#[derive(Default, Clone, Debug)]
pub struct Core {
    pub name: String,
    pub usage: f32,
    pub frequency: u64,
}

#[derive(Default, Clone, Debug)]
pub struct CpuInfo {
    pub vendor: String,
    pub brand: String,
    pub usage: f32,
    pub cores: Vec<Core>,
}

pub fn draw(frame: &mut Frame, area: Rect, cores: &[Core]) {
    let vertical = Layout::vertical([Constraint::Ratio(1, Y as u32); Y]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, X as u32); X]);
    let v = vertical.areas::<Y>(area);
    let mut areas = Vec::new();
    for a in v {
        for l in horizontal.areas::<X>(a) {
            areas.push(l);
        }
    }

    let mut a = 0;

    for c in cores {
        if a < areas.len() {
            frame.render_widget(core(c), areas[a]);
            a += 1;
        }
    }
}

fn core(core: &Core) -> Gauge<'static> {
    let block = get_block().title(format!("{}", core.name));
    Gauge::default()
        .percent(core.usage as u16)
        .label(format!("{:.2}%", core.usage))
        .block(block)
        .gauge_style(Style::default().fg(Color::Rgb(137, 180, 250)))
}
