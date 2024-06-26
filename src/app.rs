use std::{cell::RefCell, rc::Rc, sync::Arc, thread, time::Duration};

use super::runner::Runner;
use tokio::sync::Mutex;
use tui::widgets::ListState;

pub struct Status;
impl Status {
    pub fn success() -> String {
        String::from("SUCCESS")
    }
    pub fn error() -> String {
        String::from("ERROR")
    }
    pub fn critical() -> String {
        String::from("CRITICAL")
    }
    pub fn warning() -> String {
        String::from("WARNING")
    }
}

pub struct App {
    pub title: String,
    pub should_quit: Arc<Mutex<bool>>,
    pub progress: f64,
    pub taken: Arc<Mutex<usize>>,
    pub miss: Arc<Mutex<usize>>,
    pub error: usize,
    pub requests_per_seconds: Arc<Mutex<usize>>,
    pub hunt: StatefulList<String>,
    pub takens: StatefulList<String>,
    pub errors: StatefulList<String>,
    pub logs: StatefulList<(String, String)>,
    pub tabs: TabsState,
    pub enhanced_graphics: bool,
    pub runner: Rc<RefCell<Runner>>,
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: vec![],
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

impl App {
    pub fn new(title: String, runner: Rc<RefCell<Runner>>, enhanced_graphics: bool) -> App {
        let requests_per_seconds = Arc::new(Mutex::new(0));
        let taken = Arc::new(Mutex::new(0));
        let miss = Arc::new(Mutex::new(0));
        let should_quit = Arc::new(Mutex::new(false));

        let shared_rs = Arc::clone(&requests_per_seconds);
        let shared_taken = Arc::clone(&taken);
        let shared_miss = Arc::clone(&miss);
        let shared_quit = Arc::clone(&should_quit);
        thread::spawn(move || {
            while !*shared_quit.blocking_lock() {
                let miss = { (*shared_miss.blocking_lock()).clone() };
                let taken = { (*shared_taken.blocking_lock()).clone() };

                let i = miss + taken;
                thread::sleep(Duration::from_secs(1));

                let miss = { (*shared_miss.blocking_lock()).clone() };
                let taken = { (*shared_taken.blocking_lock()).clone() };
                let f = miss + taken;

                *shared_rs.blocking_lock() = f - i;
            }
        });

        App {
            title,
            should_quit,
            progress: 0.0,
            taken,
            error: 0,
            hunt: StatefulList::new(),
            takens: StatefulList::new(),
            errors: StatefulList::new(),
            logs: StatefulList::new(),
            tabs: TabsState::new(vec!["Main".to_string(), "About".to_string()]),
            enhanced_graphics,
            runner,
            requests_per_seconds,
            miss,
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                *self.should_quit.blocking_lock() = true;
            }

            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        let mut runner = self.runner.borrow_mut();

        if let Some(taken) = runner.pop_taken() {
            if self.takens.items.len() > 5 {
                self.takens.items.pop();
            }
            self.takens.items.insert(0, taken);
        }

        if let Some(error) = runner.pop_error() {
            if self.errors.items.len() > 5 {
                self.errors.items.pop();
            }
            self.errors.items.insert(0, error);
        }

        if let Some(log) = runner.pop_log() {
            if self.logs.items.len() > 5 {
                self.logs.items.pop();
            }
            self.logs.items.insert(0, log);
        }

        if let Some(hunt) = runner.pop_hunt() {
            // self.hunt.items.pop().unwrap();
            self.hunt.items.insert(0, hunt);
        }
    }
}
