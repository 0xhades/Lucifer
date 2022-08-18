use super::app::App;
use super::checker::Checker;
use super::ui;
use super::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::thread;
use std::{error::Error, time::Duration};
use std::{
    io::{stdout, Read, Write},
    time::Instant,
};
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
    Log((String, String)),
    Quit,
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
    pub fn push_log(&mut self, log: (String, String)) {
        self.log.push(log);
    }
    pub fn push_hunt(&mut self, hunt: String) {
        self.hunt.push(hunt);
    }
    pub fn push_taken(&mut self, taken: String) {
        self.taken.push(taken);
    }
    pub fn push_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    config: Config,
    runner: Rc<RefCell<Runner>>,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<AppEvent>();

    let shared_config = Arc::new(config);
    let should_quit = Arc::new(AtomicBool::new(false));
    let TakenTotal = Arc::new(AtomicUsize::new(0));
    let ErrorTotal = Arc::new(AtomicUsize::new(0));
    let MissTotal = Arc::new(AtomicUsize::new(0));
    let RS = Arc::new(AtomicUsize::new(0));

    let shared = (
        Arc::clone(&TakenTotal),
        Arc::clone(&ErrorTotal),
        Arc::clone(&MissTotal),
        Arc::clone(&RS),
        Arc::clone(&should_quit),
    );

    let checker = Checker::new(
        shared_config,
        shared.0,
        shared.1,
        shared.2,
        shared.3,
        tx,
        shared.4,
    );
    let handle = thread::spawn(move || checker.run());

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

        if let Ok(evt) = rx.try_recv() {
            let mut runner = runner.borrow_mut();
            match evt {
                AppEvent::Hunt(username) => runner.push_hunt(username),
                AppEvent::Taken(username) => runner.push_taken(username),
                AppEvent::Error(username) => runner.push_error(username),
                AppEvent::Log(log) => runner.push_log(log),
                AppEvent::Quit => app.should_quit = true,
                AppEvent::Miss(_) => (),
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let mut takens = 0;
            {
                takens = TakenTotal.load(Ordering::Relaxed).clone();
            }

            let mut errors = 0;
            {
                errors = ErrorTotal.load(Ordering::Relaxed).clone();
            }

            let mut misses = 0;
            {
                misses = MissTotal.load(Ordering::Relaxed).clone();
            }

            let mut rs = 0;
            {
                rs = RS.load(Ordering::Relaxed).clone();
            }

            app.error = errors;
            app.taken = takens;
            app.miss = misses;
            app.requests_per_seconds = rs;

            app.on_tick();
            last_tick = Instant::now();
        }

        if !app.should_quit {
            continue;
        }

        should_quit.store(true, Ordering::Release);
        if let Some(e) = handle.join().unwrap() {
            return Err(e.into());
        }
        return Ok(());
    }
}
