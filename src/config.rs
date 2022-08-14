use serde::{Deserialize, Serialize};
use std::{error::Error, time::Duration};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::titles::{ANSI_REGULAR, ANSI_SHADOW, BLOODY, LARRY3D};

#[derive(Serialize, Deserialize)]
enum Title {
    Bloody,
    Larry3D,
    AnsiShadow,
    AnsiRegular,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    title: Title,
    threads: u32,
    limit_per_thread: u32,
    timeout_request: Duration,
    timeout_connect_proxy: Duration,
}

impl Config {
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: Title::Bloody,
            threads: 15,
            limit_per_thread: 100,
            timeout_request: Duration::from_secs(5),
            timeout_connect_proxy: Duration::from_secs(5),
        }
    }
}

pub async fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let serialized_config = serde_json::to_string(config)?;

    let mut opener = OpenOptions::new();
    let mut file = opener.create(true).write(true).open("config.json").await?;

    file.write_all(serialized_config.as_bytes()).await?;

    Ok(())
}

pub async fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut opener = OpenOptions::new();
    let mut file = opener.read(true).open("config.json").await?;

    let mut raw_config = String::new();
    file.read_to_string(&mut raw_config).await?;

    let config = serde_json::from_str::<Config>(&raw_config)?;
    Ok(config)
}
