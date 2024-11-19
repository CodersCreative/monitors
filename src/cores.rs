use hw_linux::cpu::cores::{Core, Cores};
use ratatui::{layout::{Constraint, Layout, Rect}, style::{Color, Style}, widgets::{Block, Gauge}, Frame};

use crate::stats::get_block;

const Y : usize = 12;
const X : usize = 5;

pub fn draw(frame: &mut Frame, area : Rect, cores : &Cores){

    let vertical = Layout::vertical([Constraint::Ratio(1, Y as u32); Y]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, X as u32); X]);
    let v = vertical.areas::<Y>(area);
    let mut areas = Vec::new();
    for a in v {
        for l in horizontal.areas::<X>(a){
            areas.push(l);
        }
    }
    
    let mut a = 0;
    
    for c in &cores.0{
        if a < areas.len(){
            if let Some(cor) = core(c){
                frame.render_widget(cor, areas[a]);
                a += 1;
            }
        }
    }
}

fn core(core : &Core) -> Option<Gauge<'static>>{
    if let Some(name) = &core.name{
        let block = get_block().title(format!("{}", name));
        if let Some(used) = core.usage{
            return Some(Gauge::default().percent(used as u16).label(format!("{}%", used)).block(block).gauge_style(Style::default().fg(Color::Rgb(137, 180, 250))));
        }
    }

    None
}
