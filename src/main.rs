#![allow(dead_code, non_snake_case, unused_imports)]
mod API;
mod api;
mod client;
mod endpoints;
mod tests;
mod useragents;
mod utils;

#[cfg(target_family = "windows")]
mod windows;

//TODO: adding linux support

#[cfg(target_family = "unix")]
mod unix;

#[cfg(target_family = "windows")]
use windows::{raise_fd_limit, MAX_FD};

#[cfg(target_family = "unix")]
use unix::{raise_fd_limit, MAX_FD};

use std::{error::Error, time::Duration};

use api::{APIs, DataAccount, UsernameBuilder};
use client::Client;
use reqwest::Proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let max_fd = raise_fd_limit(MAX_FD).unwrap();

    Ok(())
}
