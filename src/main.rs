pub mod data;
pub mod pages;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use hw_linux::{environment::packages::PackageManagers, InfoTrait};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, Borders, TableState},
    DefaultTerminal, Frame,
};
use std::{
    io,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};
use sysinfo::{System, Users};

use crate::data::Data;

const WAIT: Duration = Duration::from_millis(1000);

fn main() -> io::Result<()> {
    let (dtx, drx) = mpsc::channel();
    thread::spawn(move || {
        let mut sys = System::new_all();
        let user = Users::new_with_refreshed_list();
        loop {
            let data = Data::new(&mut sys, &user);
            let _ = dtx.send(data);
            thread::sleep(WAIT);
            sys.refresh_all();
        }
    });
    let mut terminal = ratatui::init();
    let pms = PackageManagers::get().unwrap();
    let table = TableState::default();
    let app_result = App {
        exit: false,
        page: Page::Stats1,
        history: Vec::new(),
        table,
        drx,
        pms,
    }
    .run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(PartialEq, Eq)]
enum Page {
    Stats1,
    Stats2,
    Monitor,
    Processes,
    History,
}

pub struct App {
    exit: bool,
    page: Page,
    drx: Receiver<Data>,
    history: Vec<Data>,
    pms: PackageManagers,
    table: TableState,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let main = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]);
        let [title_bar, main_area, instruction] = main.areas(frame.area());
        frame.render_widget(
            Block::new().borders(Borders::TOP).title("Monitors"),
            title_bar,
        );

        let mut ins_txt = " ← <Left> | → <Right> | Quit <q>".to_string();

        let mut draw = |data: &Data, history: &[Data]| match self.page {
            Page::Stats1 => pages::stats::draw_page_1(frame, main_area, &data, &self.pms),
            Page::Stats2 => pages::stats::draw_page_2(frame, main_area, &data),
            Page::Monitor => pages::monitor::draw(frame, main_area, &data),
            Page::Processes => {
                pages::processes::draw(frame, main_area, &mut self.table, &data.processes);
                ins_txt.push_str(" | ↑ <Up> | ↓ <Down> | Kill <k> | DeSelect <Esc>")
            }
            Page::History => pages::history::draw(frame, main_area, &history),
        };

        if let Ok(data) = self.drx.try_recv() {
            draw(&data, &self.history);

            if self.history.len() > 99 {
                self.history.remove(0);
            }
            self.history.push(data);
        } else if let Some(data) = self.history.last().map(|x| x.clone()) {
            draw(&data, &self.history);
        }

        frame.render_widget(
            Block::new().borders(Borders::TOP).title(ins_txt),
            instruction,
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Ok(ev) = event::poll(WAIT) {
            if ev {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_event(key_event)
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.previous(),
            KeyCode::Right => self.next(),
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Esc => {
                if &self.page == &Page::Processes {
                    match self.table.selected() {
                        Some(i) => match key_event.code {
                            KeyCode::Up => self.table.select_previous(),
                            KeyCode::Down => self.table.select_next(),
                            KeyCode::Char('k') => {
                                if let Some(dp) = &self.history.last() {
                                    let sys = System::new_all();
                                    sys.processes_by_exact_name(&dp.processes[i].name)
                                        .into_iter()
                                        .for_each(|x| {
                                            x.kill();
                                        });
                                    self.table.select(None);
                                }
                            }
                            KeyCode::Esc => self.table.select(None),
                            _ => {}
                        },
                        None => self.table.select_first(),
                    };
                }
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next(&mut self) {
        self.page = match self.page {
            Page::Stats1 => Page::Stats2,
            Page::Stats2 => Page::Monitor,
            Page::Monitor => Page::History,
            Page::History => Page::Processes,
            Page::Processes => Page::Stats1,
        }
    }

    fn previous(&mut self) {
        self.page = match self.page {
            Page::Stats1 => Page::Processes,
            Page::Stats2 => Page::Stats1,
            Page::Monitor => Page::Stats2,
            Page::History => Page::Monitor,
            Page::Processes => Page::History,
        }
    }
}

pub fn get_time(seconds: u64) -> String {
    let div = |x: f64, d: f64| -> f64 { (x / d) as u64 as f64 };
    let hrs = div(seconds as f64, 3600.0);
    let mins = div(seconds as f64 - hrs * 3600.0, 60.0);
    let secs = seconds as f64 - hrs * 3600.0 - mins * 60.0;

    format!("{}:{}.{}", hrs, mins, secs)
}
