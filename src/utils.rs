use super::apis::is_valid_session;
use super::style::PrintlnError;
use crossterm::style::Color;
use std::error::Error;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::iter::StepBy;
use std::slice::Iter;

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

pub fn save_hunt(path: &str, log: &str) {
    let mut file = match OpenOptions::new().create(true).write(true).open(path) {
        Ok(f) => f,
        Err(e) => return,
    };
    file.write_all(format!("{}\n", log).as_bytes()).ok();
}

/// skip the iterator to (n).
/// Warning: Please don't use it with an iterator that used `.next()`,
/// because cannot get the current index of an iterator.
fn advance_by<T>(mut iterator: T, n: usize, length: usize) -> Option<T>
where
    T: Iterator,
{
    if n >= length {
        return None;
    }

    for i in 0..n {
        if let None = iterator.next() {
            return None;
        }
    }

    Some(iterator)
}

/// `dont_wrap:`
/// dont repeat values (wrap) **if threads count is more than the list size** and
/// stick with the full list.
pub fn split_list(
    list: Vec<String>,
    parts: usize,
    mut dont_wrap: bool,
) -> Option<(Vec<Vec<String>>, i32, bool)> {
    /*
        This algortim gonna make the RAM explode if you'll use it with a very big list
        like 15Gb. So find another way when dealing with big lists...
        This is only for small lists...

        This is one of the ways to make mutexless (for the big list) multi-threading..
        Find onther way in the future.

        There are cases that this algortim might broke and need handling:

        - Workers > list size
            ✔ The extra threads going to process random items from
            the list as long as the extra threads dosen't excced or equals the
            orginial orgnized workers. if excced or equals, warp it again.
            ✔ warp it around, make the extra threads make back around
            and start from the beginning.
            - or run the alogritm again on those extra workers.

        ✔ Workers are zero -> stop the program.
        ✔ workers == list size -> each item has its own thread.
        ✔ workers are close to the list size -> the algortim takes care of it.
        ✔ check if the list is empty first before everything.
    */

    let LIST_SIZE = list.len() as i32;
    let PARTS: i32 = parts as i32;

    let extra_workers_count: i32 = PARTS - LIST_SIZE;

    // force check
    if extra_workers_count >= PARTS {
        dont_wrap = false;
    }

    let mut emergency_n: i32 = -1;
    let worker_iters: Vec<StepBy<Iter<String>>> = (0..PARTS)
        .map(|i| advance_by(list.iter(), i as usize, LIST_SIZE as usize))
        .filter(|i| i.is_some() || !dont_wrap)
        .map(|i| {
            i.unwrap_or_else(|| {
                // handling extra workers
                emergency_n += 1;
                advance_by(list.iter(), emergency_n as usize, LIST_SIZE as usize).unwrap_or_else(
                    || {
                        emergency_n = 0;
                        advance_by(list.iter(), emergency_n as usize, LIST_SIZE as usize).unwrap()
                    },
                )
            })
            .step_by(PARTS as usize)
        })
        .collect();

    let mut lists = vec![];
    for iter in worker_iters {
        lists.push(iter.map(|s| s.clone()).collect::<Vec<String>>())
    }

    Some((lists, extra_workers_count, dont_wrap))
}
