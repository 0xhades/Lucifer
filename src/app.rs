use std::sync::Arc;

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

pub struct App<'a> {
    pub title: String,
    pub should_quit: bool,
    pub progress: f64,
    pub taken: u64,
    pub error: u64,
    pub available: StatefulList<String>,
    pub takens: StatefulList<String>,
    pub errors: StatefulList<String>,
    pub logs: StatefulList<(String, String)>,
    pub tabs: TabsState,
    pub enhanced_graphics: bool,
    pub infinte: bool,
    pub runner: &'a mut Runner,
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

impl App<'_> {
    pub fn new(title: String, runner: &mut Runner, enhanced_graphics: bool, infinte: bool) -> App {
        App {
            title,
            should_quit: false,
            progress: 0.0,
            taken: 0,
            error: 0,
            available: StatefulList::new(),
            takens: StatefulList::new(),
            errors: StatefulList::new(),
            logs: StatefulList::new(),
            tabs: TabsState::new(vec!["Main".to_string(), "About".to_string()]),
            enhanced_graphics,
            runner,
            infinte,
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
                self.should_quit = true;
            }

            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // Update progress
        self.progress += 0.001;
        if self.progress > 1.0 {
            self.progress = 0.0;
        }

        if let Some(taken) = self.runner.pop_taken() {
            self.takens.items.pop();
            self.takens.items.insert(0, taken);
        }

        if let Some(error) = self.runner.pop_error() {
            self.errors.items.pop();
            self.errors.items.insert(0, error);
        }

        if let Some(log) = self.runner.pop_log() {
            self.logs.items.pop();
            self.logs.items.insert(0, log);
        }

        if let Some(available) = self.runner.pop_available() {
            // self.available.items.pop().unwrap();
            self.available.items.insert(0, available);
        }
    }
}
