use super::app::App;
use super::ui;
use super::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{mpsc, Arc};
use std::thread;
use std::{error::Error, time::Duration};
use std::{
    io::{stdout, Read, Write},
    time::Instant,
};
use tokio::sync::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text,
    widgets::{Block, Borders, Widget},
    Terminal,
};

pub enum AppEvent {
    Hunt(String),
    Taken(String),
    Error(String),
    Miss(String),
    Log(String),
}

pub struct Runner {
    config: Config,
    hunt: Vec<String>,
    taken: Vec<String>,
    errors: Vec<String>,
    log: Vec<(String, String)>,
}

impl Runner {
    pub fn new(config: Config, previous_logs: Vec<(String, String)>) -> Self {
        Self {
            config,
            hunt: Vec::new(),
            taken: Vec::new(),
            errors: Vec::new(),
            log: previous_logs,
        }
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let config = self.config.clone();

        // warp self into reference-counting pointer
        let this = Rc::new(RefCell::new(self));
        // create app and run it
        let tick_rate = Duration::from_millis(250);
        let app = App::new(
            "Lucifer".to_string(),
            Rc::clone(&this),
            true,
            config.infinte(),
        );
        run_app(&mut terminal, app, config, this, tick_rate);

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn pop_log<'a>(&mut self) -> Option<(String, String)> {
        self.log.pop()
    }
    pub fn pop_hunt<'a>(&mut self) -> Option<String> {
        self.hunt.pop()
    }
    pub fn pop_taken<'a>(&mut self) -> Option<String> {
        self.taken.pop()
    }
    pub fn pop_error<'a>(&mut self) -> Option<String> {
        self.errors.pop()
    }
    pub fn push_log(&mut self) {}
    pub fn push_hunt(&mut self) {}
    pub fn push_taken(&mut self) {}
    pub fn push_error(&mut self) {}
}

type counter = Arc<Mutex<usize>>;

/// the actual application's logic
pub fn checker(
    ValidTotal: counter,
    TakenTotal: counter,
    ErrorTotal: counter,
    MissTotal: counter,
    HuntsTotal: counter,
    RS: counter,
) -> Option<String> {
    None
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    config: Config,
    runner: Rc<RefCell<Runner>>,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<AppEvent>();

    // The total valid attempts = (Taken + Hunts + Miss)
    let ValidTotal = Arc::new(Mutex::new(0usize));
    let TakenTotal = Arc::new(Mutex::new(0usize));
    let ErrorTotal = Arc::new(Mutex::new(0usize));
    let MissTotal = Arc::new(Mutex::new(0usize));
    let HuntsTotal = Arc::new(Mutex::new(0usize));
    let RS = Arc::new(Mutex::new(0usize));

    let (valids, takens, errors, misses, hunts, rs) = (
        Arc::clone(&ValidTotal),
        Arc::clone(&TakenTotal),
        Arc::clone(&ErrorTotal),
        Arc::clone(&MissTotal),
        Arc::clone(&HuntsTotal),
        Arc::clone(&RS),
    );
    let handle = thread::spawn(|| checker(valids, takens, errors, misses, hunts, rs));

    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => app.on_key(c),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Right => app.on_right(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            if let Some(e) = handle.join().unwrap() {
                return Err(e.into());
            }
            return Ok(());
        }
    }
}
