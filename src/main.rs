mod board;
mod team;

use std::{
    env,
    error::Error,
    io,
    sync::Mutex,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use board::Board;
use team::Team;

struct App {
    state: TableState,
    teams: Vec<Team>,
    board: Option<Mutex<Board>>,
    active: u32,
    is_running: bool,
}

impl App {
    fn new(board: Option<Mutex<Board>>) -> App {
        let mut state = TableState::default();
        state.select(None);

        App {
            state,
            board,
            active: 0,
            is_running: false,
            teams: vec![
                Team::new(1, String::from("Merlijn Hunik"), String::from("13:00")),
                Team::new(2, String::from("Jesper Klomp"), String::from("13:15")),
                Team::new(3, String::from("Kjell Albers"), String::from("13:30")),
                Team::new(4, String::from("Jeroen Groot"), String::from("13:45")),
                Team::new(
                    5,
                    String::from("Chiel van Baardwijk"),
                    String::from("14:00"),
                ),
                Team::new(6, String::from("Noud van Bohemen"), String::from("14:15")),
                Team::new(7, String::from("Lucas Boogaart"), String::from("15:00")),
                Team::new(8, String::from("Sake de Vries"), String::from("15:15")),
                Team::new(9, String::from("Tim Huysse"), String::from("15:30")),
                Team::new(10, String::from("Lars de Nijs"), String::from("15:45")),
                Team::new(11, String::from("Matt Molenaar"), String::from("16:00")),
                Team::new(
                    12,
                    String::from("Sebastiaan van Paassen"),
                    String::from("16:15"),
                ),
                Team::new(13, String::from("Ries Meijssen"), String::from("16:30")),
                Team::new(14, String::from("Isabel Hille"), String::from("16:45")),
                Team::new(15, String::from("Julia Ansems"), String::from("17:00")),
            ],
        }
    }
    pub fn next(&mut self) {
        if self.teams.is_empty() {
            return;
        }

        let selected = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        let new = if selected >= self.teams.len() - 1 {
            0
        } else {
            selected + 1
        };

        self.state.select(Some(new));
    }

    pub fn previous(&mut self) {
        if self.teams.is_empty() {
            return;
        }

        let selected = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        let new = if selected <= 0 {
            self.teams.len() - 1
        } else {
            selected - 1
        };

        self.state.select(Some(new));
    }

    fn start_stop_current(&mut self) {
        let id = match self.state.selected() {
            Some(i) => i,
            None => return,
        };

        if self.is_running {
            if self.active == self.teams[id].id {
                let time = self
                    .teams
                    .iter()
                    .find(|&t| t.id == self.active)
                    .unwrap()
                    .get_time()
                    .replace(":", "");
                if time.len() == 6 {
                    if let Some(board) = &self.board {
                        let mut board = board.lock().unwrap();
                        board.write(1, format!(" {}", time));
                    }
                }

                self.teams[id].start_stop_timer();
                self.is_running = false;
            }
        } else {
            self.teams[id].start_stop_timer();
            self.active = self.teams[id].id;
            self.is_running = true;
        }
    }

    fn reset_current(&mut self) {
        let selected = match self.state.selected() {
            Some(i) => i,
            None => return,
        };

        self.teams[selected].reset_time();
    }

    fn create_new(&mut self) {
        println!("creating new");
    }

    fn on_tick(&mut self) {
        self.teams.sort();

        if let Some(team) = self.teams.first() {
            let time = team.get_time().replace(":", "");
            if time.len() == 6 {
                if let Some(board) = &self.board {
                    let mut board = board.lock().unwrap();
                    board.write(3, format!(" {}", time));
                }
            }
        }

        if self.is_running {
            let time = self
                .teams
                .iter()
                .find(|&t| t.id == self.active)
                .unwrap()
                .get_time()
                .replace(":", "");
            if time.len() == 6 {
                if let Some(board) = &self.board {
                    let mut board = board.lock().unwrap();
                    board.write(1, format!(" {}", time));
                }
            }
        }

        if let Some(board) = &self.board {
            let mut board = board.lock().unwrap();
            board.tick();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let port = env::args().nth(1);
    let board = if let Some(port) = port {
        let mut board = Board::new(port);
        board.write(0, " TIJD ".to_string());
        board.write(2, "TOPTIJD".to_string());

        Some(Mutex::new(board))
    } else {
        println!("WARN: Port not supplied, running headless.");
        None
    };

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(59);
    let app = App::new(board);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => app.create_new(),
                    KeyCode::Char('c') => app.reset_current(),
                    KeyCode::Enter => app.start_stop_current(),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(2), Constraint::Length(3)].as_ref())
        .split(size);

    let help =
        Paragraph::new("q: Exit | n: Nieuwe Deelnemer | Space: Start/Stop tijd | c: Reset Tijd")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Help")
                    .border_type(BorderType::Plain),
            );
    f.render_widget(help, chunks[1]);

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["Group Nummer", "Team Captain", "Starttijd", "Racetijd"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Blue)));
    let header = Row::new(header_cells).style(Style::default()).height(1);

    let rows = app.teams.iter().map(|item| item.into());

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Deelnemers"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Min(10),
            Constraint::Min(20),
            Constraint::Min(10),
            Constraint::Min(20),
        ]);

    f.render_stateful_widget(t, chunks[0], &mut app.state);
}
