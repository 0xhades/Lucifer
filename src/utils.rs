use super::apis::is_valid_session;
use super::style::PrintlnError;
use crossterm::style::Color;
use std::error::Error;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Read, Write};

pub fn handle<T, E>(result: Result<T, E>, message: &str, cast_error: bool, quit: bool) -> Option<T>
where
    E: Display,
{
    match result {
        Ok(t) => Some(t),
        Err(e) => {
            PrintlnError(
                {
                    if cast_error {
                        format!("{}: {}", message, e)
                    } else {
                        message.to_string()
                    }
                },
                quit,
                Color::Red,
                Color::Cyan,
            )
            .ok();
            None
        }
    }
}

pub fn is_valid_proxy(proxy: &str) -> Option<String> {
    if proxy.contains(":") {
        let splited = proxy.split(":").collect::<Vec<&str>>();
        if splited.len() >= 2 {
            return Some(format!(
                "{}:{}",
                splited.get(0)?.trim(),
                splited.get(1)?.trim()
            ));
        }
    }

    None
}

pub fn load_usernames(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    let lines = raw
        .lines()
        .into_iter()
        .filter(|s| s.len() != 0)
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    Ok(lines)
}

pub fn load_proxies(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    let lines = raw
        .lines()
        .into_iter()
        .map(|s| is_valid_proxy(s))
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect::<Vec<String>>();

    Ok(lines)
}

pub fn load_sessions(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    let lines = raw
        .lines()
        .into_iter()
        .map(|s| is_valid_session(s))
        .filter(|s| s.is_some())
        .map(|s| s.unwrap().trim().to_string())
        .collect::<Vec<String>>();

    Ok(lines)
}

pub fn save_log(path: &str, log: &str) {
    let mut file = match OpenOptions::new().write(true).open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    file.write_all(format!("{}\n", log).as_bytes()).ok();
}
