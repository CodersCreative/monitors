use crate::{data::Process, get_time};
use ratatui::{
    layout::{Constraint::Fill, Rect},
    style::{Style, Stylize},
    widgets::{HighlightSpacing, Row, Table, TableState},
    Frame,
};

pub fn draw(frame: &mut Frame, area: Rect, table: &mut TableState, processes: &Vec<Process>) {
    let mut rows = Vec::new();
    for p in processes {
        rows.push(process(p));
    }

    let widths = [
        Fill(1),
        Fill(2),
        Fill(2),
        Fill(1),
        Fill(1),
        Fill(2),
        Fill(7),
    ];
    frame.render_stateful_widget(
        Table::new(rows, widths)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_blue())
            .header(Row::new(vec![
                "PID", "USER", "MEM (Mb)", "CPU%", "MEM%", "TIME", "COMMAND",
            ])),
        area,
        table,
    );
}

fn process<'a>(process: &'a Process) -> Row<'a> {
    Row::new(vec![
        process.pid.to_string(),
        process.user.clone(),
        format!("{:.1}", process.memory as f64 / 1024_f64 / 1024_f64),
        format!("{:.1}", process.cpu),
        format!("{:.1}", process.memory as f64 / process.total_m as f64),
        get_time(process.run_time),
        process.command.clone(),
    ])
}
