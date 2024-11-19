use std::ffi::{OsStr, OsString};

use ratatui::{layout::{Constraint::{self, Fill, Length}, Rect}, style::{Style, Stylize}, widgets::{HighlightSpacing, Row, Table, TableState}, Frame};
use sysinfo::{Pid, Uid};

use crate::get_time;


pub struct Process{
    pub user : String,
    pub name : OsString,
    pub pid : Pid,
    pub memory : u64,
    pub cpu : f32,
    pub command : String,
    pub run_time : u64,
    pub total_m : u64,
}

pub fn draw(frame: &mut Frame, area : Rect, table : &mut TableState, processes : &Vec<Process>){
    let mut rows = Vec::new();
    for p in processes{
        rows.push(process(p));
    }

    let widths = [Fill(1), Fill(2), Fill(2), Fill(1), Fill(1), Fill(2), Fill(7)];
    frame.render_stateful_widget(Table::new(rows, widths)
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol(">>")
        .row_highlight_style(Style::new().on_blue())
        .header(Row::new(vec!["PID", "USER", "MEM (Mb)", "CPU%", "MEM%", "TIME", "COMMAND"])), area, table);
}

fn process(process : &Process) -> Row{
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
