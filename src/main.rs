#![allow(dead_code, non_snake_case, unused_imports, unreachable_code)]
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

use api::{APIs, DataAccount, UsernameBuilder};
use clap::{Arg, Command};
use client::Client;
use config::{load_config, save_config, Config};
use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use reqwest::Proxy;
use std::path::Path;
use std::{error::Error, time::Duration};
use std::{
    io::{stdout, Read, Write},
    process::exit,
};
use style::{
    clear, PrintColorful, PrintlnColorful, PrintlnColorfulPlus, PrintlnError, PrintlnErrorQuit,
    PrintlnSuccess,
};
use utils::handle;

/*
TODO:
    - Settings [✔]
    - Better error handling [✔]
    - Command Line options [✔]
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    clear()?;

    PrintlnColorful(titles::LARRY3D, Color::Red)?;
    PrintColorful("Coder: ", Color::Cyan)?;
    PrintlnColorful("#0xhades", Color::Yellow)?;

    let mut config = Config::default();

    let matches = Command::new("lucifer")
        .about("An instagram checker for hunting instagram's rare usernames.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("config")
                .about("Run with user-input-config then saving it and run.")
                .arg_required_else_help(true)
                .args(&[
                    Arg::with_name("threads")
                        .short('w')
                        .long("threads")
                        .help("The amount of threads for each asynchronous pool.")
                        .value_name("NUMBER")
                        .default_value("15")
                        .takes_value(true),
                    Arg::with_name("limit")
                        .short('l')
                        .long("limit")
                        .help("Sets a tasks limit for each thread.")
                        .value_name("NUMBER")
                        .default_value("100")
                        .takes_value(true),
                    Arg::with_name("connect")
                        .short('c')
                        .long("connect")
                        .help("Sets the proxies connect timeout in secs.")
                        .value_name("DURATION")
                        .default_value("5")
                        .takes_value(true),
                    Arg::with_name("request")
                        .short('r')
                        .long("request")
                        .help("Sets the requests (writing, reading) timeout in secs.")
                        .value_name("DURATION")
                        .default_value("5")
                        .takes_value(true),
                    Arg::with_name("title")
                        .short('t')
                        .long("title")
                        .help("Sets the program title style [bloody, larry, regular, shadow].")
                        .value_name("STYLE")
                        .default_value("bloody")
                        .takes_value(true),
                    Arg::with_name("proxy")
                        .short('p')
                        .long("proxy")
                        .help("The path to the proxy file.")
                        .value_name("FILE")
                        .default_value("$")
                        .takes_value(true),
                    Arg::with_name("username")
                        .short('u')
                        .long("username")
                        .help("The path to the usernames file.")
                        .value_name("FILE")
                        .default_value("$")
                        .takes_value(true),
                    Arg::with_name("type")
                        .short('s')
                        .long("type")
                        .help("The type of the proxies [socks5, socks4, http, https].")
                        .value_name("TYPE")
                        .default_value("http")
                        .takes_value(true),
                ]),
        )
        .subcommand(Command::new("default").about("Run with the default config."))
        .subcommand(
            Command::new("load")
                .about("Load config file and run.")
                .args(&[
                    Arg::with_name("path")
                        .short('p')
                        .long("path")
                        .help("The path to the config file.")
                        .value_name("FILE")
                        .default_value("config.json")
                        .required(false)
                        .takes_value(true),
                    Arg::with_name("local")
                        .short('l')
                        .long("local")
                        .help("Load the local config file.")
                        .required(false),
                ]),
        )
        .get_matches();

    let mut connect_timeout = Duration::from_secs(5);
    let mut request_timeout = Duration::from_secs(5);
    let mut limit = 100;
    let mut threads = 15;
    let mut path = ".";
    let mut title = "larry";
    let mut proxies_type = "http";
    let mut proxies_path = "$";
    let mut username_path = "$";

    let mut user_input_config = false;
    let mut default_config_path = false;

    match matches.subcommand() {
        Some(("config", sub_matches)) => {
            user_input_config = true;
            connect_timeout = Duration::from_secs(
                sub_matches
                    .get_one::<String>("connect")
                    .expect("required")
                    .clone()
                    .parse::<u64>()
                    .unwrap(),
            );

            request_timeout = Duration::from_secs(
                sub_matches
                    .get_one::<String>("request")
                    .expect("required")
                    .clone()
                    .parse::<u64>()
                    .unwrap(),
            );

            limit = sub_matches
                .get_one::<String>("limit")
                .expect("required")
                .clone()
                .parse::<u32>()
                .unwrap();
            threads = sub_matches
                .get_one::<String>("threads")
                .expect("required")
                .clone()
                .parse::<u32>()
                .unwrap();
            title = sub_matches.get_one::<String>("title").expect("required");

            let s = sub_matches.get_one::<String>("proxy").expect("required");
            if Path::new(s).is_file() {
                proxies_path = s;
            } else {
                PrintlnErrorQuit(format!("Couldn't load: {}", s), Color::Red, Color::Cyan);
            }

            let s = sub_matches.get_one::<String>("username").expect("required");
            if Path::new(s).is_file() {
                username_path = s;
            } else {
                PrintlnErrorQuit(format!("Couldn't load: {}", s), Color::Red, Color::Cyan);
            }

            proxies_type = sub_matches.get_one::<String>("type").expect("required");
        }
        Some(("load", sub_matches)) => {
            if sub_matches.is_present("path") {
                let s = sub_matches.get_one::<String>("path").expect("required");
                if Path::new(s).is_file() {
                    path = s;
                } else {
                    PrintlnErrorQuit(format!("Couldn't load: {}", s), Color::Red, Color::Cyan);
                }
            } else {
                default_config_path = true;
                path = "config.json"
            }
        }
        _ => (),
    }

    if default_config_path || path == "." {
        if !Path::new("config.json").is_file() {
            PrintlnError(
                format!("Couldn't load: config.json. Creating new default config..",),
                false,
                Color::Red,
                Color::Cyan,
            )?;
            config = Config::default();
            if let Err(e) = save_config(&config).await {
                PrintlnErrorQuit(
                    format!("Couldn't save the config to file: {}", e),
                    Color::Red,
                    Color::Cyan,
                );
            };
        }
    } else {
        config = match load_config(path).await {
            Ok(c) => c,
            Err(e) => PrintlnErrorQuit(
                format!(
                    "Couldn't load the config from file: {}, reason: {}",
                    path, e
                ),
                Color::Red,
                Color::Cyan,
            ),
        };
    }

    if user_input_config {
        config = Config::new(
            title,
            proxies_type,
            proxies_path,
            threads,
            limit,
            request_timeout,
            connect_timeout,
            username_path,
        );

        if let Err(e) = save_config(&config).await {
            PrintlnErrorQuit(
                format!("Couldn't save the config to file: {}", e),
                Color::Red,
                Color::Cyan,
            );
        };
    }

    if !config.resolve_proxy_path() {
        PrintlnErrorQuit(
            format!("Couldn't load any proxy file: {}", config.proxy_path()),
            Color::Red,
            Color::Cyan,
        );
    };

    if !config.resolve_username_path() {
        PrintlnErrorQuit(
            format!(
                "Couldn't load any username file: {}",
                config.username_path()
            ),
            Color::Red,
            Color::Cyan,
        );
    };

    PrintlnColorfulPlus("Loading..", Color::Cyan, Color::Red)?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    clear()?;

    PrintlnColorful(config.title(), Color::Red)?;
    PrintColorful("Coder: ", Color::Cyan)?;
    PrintlnColorful("#0xhades", Color::Yellow)?;

    if let Some(new_fd) = handle(
        raise_fd_limit(MAX_FD),
        "Couldn't change the FD limit\n",
        false,
        false,
    ) {
        PrintlnSuccess(
            format!("Changed the FD limit to {} successfully", new_fd).as_str(),
            Color::Cyan,
            Color::Red,
        )?;
    }

    PrintlnColorfulPlus("Welcome to lucifer!", Color::Cyan, Color::Red)?;

    Ok(())
}
