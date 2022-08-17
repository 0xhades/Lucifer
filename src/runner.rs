use super::app::App;
use super::ui;
use super::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::sync::Arc;
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

pub struct Runner {
    config: Config,
    available: Arc<Mutex<Vec<String>>>,
    taken: Arc<Mutex<Vec<String>>>,
    errors: Arc<Mutex<Vec<String>>>,
    log: Arc<Mutex<Vec<(String, String)>>>,
}

impl Runner {
    pub fn new(config: Config, previous_logs: Vec<(String, String)>) -> Self {
        Self {
            config,
            available: Arc::new(Mutex::new(Vec::new())),
            taken: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
            log: Arc::new(Mutex::new(previous_logs)),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        //TODO: run it in background
        // create app and run it
        let tick_rate = Duration::from_millis(250);
        let app = App::new("Lucifer".to_string(), self, true, self.config.infinte());
        run_app(&mut terminal, app, tick_rate)?;

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
        self.log.blocking_lock().pop()
    }
    pub fn pop_available<'a>(&mut self) -> Option<String> {
        self.available.blocking_lock().pop()
    }
    pub fn pop_taken<'a>(&mut self) -> Option<String> {
        self.taken.blocking_lock().pop()
    }
    pub fn pop_error<'a>(&mut self) -> Option<String> {
        self.errors.blocking_lock().pop()
    }
    pub fn push_log(&mut self) {}
    pub fn push_available(&mut self) {}
    pub fn push_taken(&mut self) {}
    pub fn push_error(&mut self) {}
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
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
            return Ok(());
        }
    }
}
