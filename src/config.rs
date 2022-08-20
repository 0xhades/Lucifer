use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::exit;
use std::{error::Error, time::Duration};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::style::PrintlnError;
use super::titles::{ANSI_REGULAR, ANSI_SHADOW, BLOODY, LARRY3D};

#[derive(Serialize, Deserialize, Clone)]
enum Title {
    Bloody,
    Larry3D,
    AnsiShadow,
    AnsiRegular,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ProxyType {
    HTTP,
    SOCKS5,
    HTTPS,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    title: Title,
    threads: u32,
    limit_per_thread: u32,
    timeout_request: Duration,
    timeout_connect_proxy: Duration,
    proxy_type: ProxyType,
    proxy_path: String,
    username_path: String,
    sessions_path: String,
}

impl Config {
    pub fn new(
        title: &str,
        proxy_type: &str,
        proxy_path: &str,
        threads: u32,
        limit_per_thread: u32,
        timeout_request: Duration,
        timeout_connect_proxy: Duration,
        username_path: &str,
        sessions_path: &str,
    ) -> Self {
        let proxy_type = match proxy_type.trim().to_lowercase().as_str() {
            "http" => ProxyType::HTTP,
            "socks5" => ProxyType::SOCKS5,
            "https" => ProxyType::HTTPS,
            _ => {
                PrintlnError(
                    format!("Invalid proxy type: {}", proxy_type),
                    true,
                    Color::Red,
                    Color::Cyan,
                )
                .unwrap();
                exit(1)
            }
        };

        let title = match title.trim().to_lowercase().as_str() {
            "bloody" => Title::Bloody,
            "larry" => Title::Larry3D,
            "regular" => Title::AnsiRegular,
            "shadow" => Title::AnsiShadow,
            _ => {
                PrintlnError(
                    format!("Invalid title style: {}", title),
                    true,
                    Color::Red,
                    Color::Cyan,
                )
                .unwrap();
                exit(1)
            }
        };

        Self {
            title,
            threads,
            limit_per_thread,
            timeout_request,
            timeout_connect_proxy,
            proxy_type,
            proxy_path: proxy_path.to_string(),
            username_path: username_path.to_string(),
            sessions_path: sessions_path.to_string(),
        }
    }

    pub fn title(&self) -> &str {
        match self.title {
            Title::Bloody => BLOODY,
            Title::AnsiRegular => ANSI_REGULAR,
            Title::Larry3D => LARRY3D,
            Title::AnsiShadow => ANSI_SHADOW,
        }
    }

    pub fn threads(&self) -> u32 {
        self.threads
    }

    pub fn limit_per_thread(&self) -> u32 {
        self.limit_per_thread
    }

    pub fn timeout_request(&self) -> Duration {
        self.timeout_request
    }

    pub fn timeout_connect_proxy(&self) -> Duration {
        self.timeout_connect_proxy
    }

    pub fn proxy_type(&self) -> ProxyType {
        self.proxy_type.clone()
    }

    pub fn proxy_path(&self) -> String {
        self.proxy_path.clone()
    }

    pub fn session_path(&self) -> String {
        self.sessions_path.clone()
    }

    pub fn username_path(&self) -> String {
        self.username_path.clone()
    }

    pub fn resolve_proxy_path(&mut self) -> bool {
        if self.proxy_path == "$" {
            let files = ["proxy.txt", "proxies.txt", "p.txt", "ip.txt", "ips.txt"].into_iter();
            let exists = files
                .filter(|f| Path::new(f).is_file())
                .collect::<Vec<&str>>();

            if exists.len() != 0 {
                self.proxy_path = exists.get(0).unwrap().to_string();
                return true;
            }
            return false;
        }

        if Path::new(&self.proxy_path).is_file() {
            return true;
        }

        false
    }

    pub fn resolve_username_path(&mut self) -> bool {
        if self.username_path == "$" {
            let files = [
                "username.txt",
                "usernames.txt",
                "u.txt",
                "users.txt",
                "user.txt",
            ]
            .into_iter();
            let exists = files
                .filter(|f| Path::new(f).is_file())
                .collect::<Vec<&str>>();

            if exists.len() != 0 {
                self.username_path = exists.get(0).unwrap().to_string();
                return true;
            }
            return false;
        }

        if Path::new(&self.username_path).is_file() {
            return true;
        }

        false
    }

    pub fn resolve_sessions_path(&mut self) -> bool {
        if self.sessions_path == "$" {
            let files = [
                "session.txt",
                "sessions.txt",
                "s.txt",
                "sessionid.txt",
                "sessionids.txt",
            ]
            .into_iter();
            let exists = files
                .filter(|f| Path::new(f).is_file())
                .collect::<Vec<&str>>();

            if exists.len() != 0 {
                self.sessions_path = exists.get(0).unwrap().to_string();
                return true;
            }
            return false;
        }

        if Path::new(&self.sessions_path).is_file() {
            return true;
        }

        false
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: Title::Larry3D,
            threads: 15,
            limit_per_thread: 100,
            timeout_request: Duration::from_secs(5),
            timeout_connect_proxy: Duration::from_secs(5),
            proxy_path: String::from("$"),
            proxy_type: ProxyType::HTTP,
            username_path: String::from("$"),
            sessions_path: String::from("$"),
        }
    }
}

pub async fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let serialized_config = serde_json::to_string(config)?;

    let mut opener = OpenOptions::new();
    let mut file = opener
        .create(true)
        .truncate(true)
        .write(true)
        .open("config.json")
        .await?;

    file.write_all(serialized_config.as_bytes()).await?;

    Ok(())
}

pub async fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let mut opener = OpenOptions::new();
    let mut file = opener.read(true).write(true).open(path).await?;

    let mut raw_config = String::new();
    file.read_to_string(&mut raw_config).await?;
    file.shutdown().await?;

    if raw_config.ends_with("}}") {
        let mut opener = OpenOptions::new();
        let mut file = opener
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;
        raw_config.remove(raw_config.len() - 1);
        file.write_all(raw_config.as_bytes()).await?;
    }

    let config = serde_json::from_str::<Config>(&raw_config)?;
    Ok(config)
}
