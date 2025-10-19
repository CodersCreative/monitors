use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::Gauge,
    Frame,
};

use crate::{
    data::{Core, CpuInfo, Data, Memory},
    pages::get_block,
};

const Y: usize = 12;
const X: usize = 3;

pub fn draw(frame: &mut Frame, area: Rect, data: &Data) {
    let vertical = Layout::vertical([Constraint::Ratio(1, Y as u32); Y]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, X as u32); X]);
    let v = vertical.areas::<Y>(area);
    let mut areas = Vec::new();

    for a in v {
        for l in horizontal.areas::<X>(a) {
            areas.push(l);
        }
    }

    let mut widgets = vec![cpu(&data.cpu), memory(&data.memory), swap(&data.memory)];
    for c in data.cpu.cores.iter() {
        widgets.push(core(c));
    }

    for (i, widget) in widgets.into_iter().enumerate() {
        if i < areas.len() {
            frame.render_widget(widget, areas[i]);
        }
    }
}

fn cpu(cpu: &CpuInfo) -> Gauge<'static> {
    let block = get_block().title("CPU");
    Gauge::default()
        .percent(cpu.usage as u16)
        .label(format!("{:.2}%", cpu.usage))
        .block(block)
        .gauge_style(Style::default().fg(Color::Green))
}

fn memory(mem: &Memory) -> Gauge<'static> {
    let block = get_block().title("Memory");
    Gauge::default()
        .percent((mem.used_mem as f64 / mem.total_mem as f64 * 100.0) as u16)
        .label(format!(
            "{:.2}%",
            (mem.used_mem as f64 / mem.total_mem as f64 * 100.0)
        ))
        .block(block)
        .gauge_style(Style::default().fg(Color::Green))
}

fn swap(mem: &Memory) -> Gauge<'static> {
    let block = get_block().title("Swap");
    Gauge::default()
        .percent((mem.used_swap as f64 / mem.total_swap as f64 * 100.0) as u16)
        .label(format!(
            "{:.2}%",
            (mem.used_swap as f64 / mem.total_swap as f64 * 100.0)
        ))
        .block(block)
        .gauge_style(Style::default().fg(Color::Green))
}

fn core(core: &Core) -> Gauge<'static> {
    let block = get_block().title(format!("{}", core.name));
    Gauge::default()
        .percent(core.usage as u16)
        .label(format!("{:.2}%", core.usage))
        .block(block)
        .gauge_style(Style::default().fg(Color::Blue))
}
