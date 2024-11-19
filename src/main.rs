
pub mod stats;
pub mod cores;
pub mod processes;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hw_linux::{cpu::cores::{Core, Cores}, environment::packages::PackageManagers, InfoTrait};
use ratatui::{layout::{Constraint, Layout}, widgets::{Block, Borders, TableState}, DefaultTerminal, Frame};
use sysinfo::{System, Users};
use std::{io, sync::mpsc::{self, Receiver}, thread, time::Duration};
use processes::Process;

const WAIT : Duration = Duration::from_millis(1000);

fn main() -> io::Result<()> {
    let (ctx, crx) = mpsc::channel();
    let (ptx, prx) = mpsc::channel();
    thread::spawn(move || {
        let mut sys = System::new_all();
        let user = Users::new_with_refreshed_list();
        loop{
            let c = Cores(sys.cpus().iter().map(|x| Core{name : Some(x.name().to_string()), usage : Some((x.cpu_usage()) as u64)}).collect::<Vec<Core>>());
            let cpus = c.0.len();
            let _ = ctx.send(c).unwrap();
            let total_m = sys.total_memory();
            let mut p = sys.processes().iter().map(|(p, x)|
                Process{
                    pid : *p,
                    name : x.name().to_owned(), 
                    user : user.get_user_by_id(x.user_id().unwrap()).unwrap().name().to_string(),
                    command : match x.cmd().first(){
                        Some(x) => x.to_str().unwrap().to_string(),
                        None => x.name().to_str().unwrap().to_string(),
                    },
                    cpu : x.cpu_usage() / cpus as f32,
                    memory : x.memory(),
                    run_time : x.run_time(),
                    total_m,
                }
            ).collect::<Vec<Process>>();
            p.sort_by(|a, b|b.cpu.total_cmp(&a.cpu));
            let _ = ptx.send(p);
            thread::sleep(WAIT);
            sys.refresh_all();
        }
    });
    let mut terminal = ratatui::init();
    let pms = PackageManagers::get().unwrap();
    let table = TableState::default();
    let app_result = App{exit : false, page : Page::Main, cp : None, pp : None,table, crx, prx, pms}.run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(PartialEq, Eq)]
enum Page{
    Main,
    Cores,
    Processes,
}
pub struct App {
    exit: bool, 
    page : Page, 
    crx : Receiver<Cores>, 
    cp : Option<Cores>,
    prx : Receiver<Vec<Process>>,
    pp : Option<Vec<Process>>,
    pms : PackageManagers, 
    table : TableState
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
            Block::new().borders(Borders::TOP).title("RSInfo"),
            title_bar,
        );

        let mut ins_txt =  " ← <Left> | → <Right> | Quit <q>".to_string();
        match self.page{
            Page::Main => stats::draw(frame, main_area, &self.pms),
            Page::Cores => {
                if let Ok(c) = self.crx.try_recv() {
                    cores::draw(frame, main_area, &c);
                    self.cp = Some(c);
                }else if let Some(c) = &self.cp{
                    cores::draw(frame, main_area, c);
                }
            },
            Page::Processes => {
                if self.table.selected() == None{
                    if let Ok(p) = self.prx.try_recv(){
                        processes::draw(frame, main_area, &mut self.table, &p);
                        self.pp = Some(p);
                    }
                }else if let Some(p) = &self.pp{
                    processes::draw(frame, main_area, &mut self.table, p);
                }
                ins_txt.push_str(" | ↑ <Up> | ↓ <Down> | Kill <k> | DeSelect <Esc>")
            },
        }

        frame.render_widget(
            Block::new().borders(Borders::TOP).title(ins_txt),
            instruction,
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Ok(ev) = event::poll(WAIT){
            if ev{
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
                if &self.page == &Page::Processes{
                    match self.table.selected(){
                        Some(i) => {
                            match key_event.code{
                                KeyCode::Up => self.table.select_previous(),
                                KeyCode::Down => self.table.select_next(),
                                KeyCode::Char('k') => {
                                    if let Some(pp) = &self.pp{
                                        let sys = System::new_all();
                                        sys.processes_by_exact_name(&pp[i].name).into_iter().for_each(|x| {x.kill();});
                                        self.table.select(None);
                                    }
                                },
                                KeyCode::Esc => self.table.select(None),
                                _ => {}
                            }
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

    fn previous(&mut self) {
        self.page = match self.page{
            Page::Main => Page::Processes,
            Page::Cores => Page::Main,
            Page::Processes => Page::Cores,
        }
    }

    fn next(&mut self) {
        self.page = match self.page{
            Page::Main => Page::Cores,
            Page::Cores => Page::Processes,
            Page::Processes => Page::Main,
        }
    }
}

pub fn get_time(seconds : u64) -> String {
    let div = |x : f64, d : f64| -> f64{
        (x / d) as u64 as f64
    };
    let hrs = div(seconds as f64, 3600.0);
    let mins = div(seconds as f64 - hrs * 3600.0, 60.0);
    let secs = seconds as f64 - hrs * 3600.0 - mins * 60.0;

    format!("{}:{}.{}", hrs, mins, secs)
}
