use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Chart, Dataset, GraphType},
    Frame,
};

use crate::{data::Data, pages::get_block};

const Y: usize = 3;
const X: usize = 1;

#[derive(Debug)]
struct HistoryData(pub Vec<(f64, f64)>);

impl HistoryData {
    fn from_cores(history: &[Data]) -> Vec<Self> {
        let mut lst: Vec<Vec<(f64, f64)>> = Vec::new();

        for (x, data) in history.iter().enumerate() {
            for (y, core) in data.cpu.cores.iter().enumerate() {
                if let Some(d) = lst.get_mut(y) {
                    d.push((x as f64, core.usage as f64));
                } else {
                    lst.push(vec![(x as f64, core.usage as f64)]);
                }
            }
        }

        lst.into_iter().map(|x| HistoryData(x)).collect()
    }

    fn from_cpu(history: &[Data]) -> Self {
        Self(
            history
                .iter()
                .enumerate()
                .map(|x| (x.0 as f64, x.1.cpu.usage as f64))
                .collect(),
        )
    }

    fn from_mem(history: &[Data]) -> (Self, Self) {
        (
            Self(
                history
                    .iter()
                    .enumerate()
                    .map(|x| {
                        (
                            x.0 as f64,
                            x.1.memory.used_mem as f64 / x.1.memory.total_mem as f64 * 100.0,
                        )
                    })
                    .collect(),
            ),
            Self(
                history
                    .iter()
                    .enumerate()
                    .map(|x| {
                        (
                            x.0 as f64,
                            x.1.memory.used_swap as f64 / x.1.memory.total_swap as f64 * 100.0,
                        )
                    })
                    .collect(),
            ),
        )
    }
}

pub fn draw(frame: &mut Frame, area: Rect, history: &[Data]) {
    let vertical = Layout::vertical([Constraint::Ratio(1, Y as u32); Y]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, X as u32); X]);
    let v = vertical.areas::<Y>(area);
    let mut areas = Vec::new();

    for a in v {
        for l in horizontal.areas::<X>(a) {
            areas.push(l);
        }
    }

    let cores_data = HistoryData::from_cores(history);
    let cpu_data = HistoryData::from_cpu(history);
    let mem_data = HistoryData::from_mem(history);

    let widgets = vec![cpu(&cpu_data), cores(&cores_data), memory(&mem_data)];

    for (i, widget) in widgets.into_iter().enumerate() {
        if i < areas.len() {
            frame.render_widget(widget, areas[i]);
        }
    }
}

fn default_chart<'a>(data: Vec<Dataset<'a>>) -> Chart<'a> {
    Chart::new(data)
        .x_axis(Axis::default().title("Time").bounds([0.0, 100.0]))
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .labels(["0%", "25%", "50%", "75%", "100%"]),
        )
}

fn cpu<'a>(data: &'a HistoryData) -> Chart<'a> {
    let block = get_block().title("CPU");
    default_chart(vec![Dataset::default()
        .name("CPU")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::new().blue())
        .data(&data.0)])
    .legend_position(None)
    .block(block)
}

fn memory<'a>(data: &'a (HistoryData, HistoryData)) -> Chart<'a> {
    let block = get_block().title("Memory");
    default_chart(vec![
        Dataset::default()
            .name("Memory")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().blue())
            .data(&data.0 .0),
        Dataset::default()
            .name("Swap")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().yellow())
            .data(&data.1 .0),
    ])
    .legend_position(Some(ratatui::widgets::LegendPosition::BottomLeft))
    .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)))
    .block(block)
}

fn cores<'a>(data: &'a [HistoryData]) -> Chart<'a> {
    let block = get_block().title("Cores");
    default_chart(
        data.into_iter()
            .enumerate()
            .map(|(i, x)| {
                Dataset::default()
                    .name(format!("cpu{}", i))
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::new().fg(ratatui::style::Color::Indexed(i as u8)))
                    .data(&x.0)
            })
            .collect(),
    )
    .legend_position(Some(ratatui::widgets::LegendPosition::BottomLeft))
    .hidden_legend_constraints((
        Constraint::Ratio(1, data[0].0.len() as u32),
        Constraint::Ratio(1, data[0].0.len() as u32),
    ))
    .block(block)
}
