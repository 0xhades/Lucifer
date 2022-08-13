#![allow(dead_code, non_snake_case, unused_imports)]
mod API;
mod api;
mod client;
mod endpoints;
mod tests;
mod useragents;
mod utils;

use std::{error::Error, time::Duration};

use api::{APIs, DataAccount, UsernameBuilder};
use client::Client;
use reqwest::Proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
