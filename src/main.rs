#![allow(dead_code, non_snake_case, unused_imports)]
mod API;
mod api;
mod client;
mod config;
mod endpoints;
mod style;
mod tests;
mod titles;
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
use config::{load_config, save_config, Config};
use reqwest::Proxy;

use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Read, Write};
use style::{clear, PrintColorful, PrintColorless, PrintError, PrintSuccess};
use utils::handle;

/*
TODO:
    - Settings [✔]
    - Better error handling [✔]
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match load_config().await {
        Ok(t) => t,
        _ => {
            PrintColorful(
                "Found no config, creating a new config with default values\n",
                Color::Magenta,
            )?;
            save_config(&Config::default()).await?;
            tokio::time::sleep(Duration::from_millis(1500)).await;
            Config::default()
        }
    };

    clear()?;

    PrintColorful(config.title(), Color::Red)?;
    PrintColorful("Coder: #0xhades\n", Color::Red)?;

    if let Some(new_fd) = handle(
        raise_fd_limit(MAX_FD),
        "Couldn't change the FD limit\n",
        false,
        false,
    ) {
        PrintSuccess(format!("Changed the FD limit to {} successfully\n", new_fd).as_str())?;
    }

    Ok(())
}
