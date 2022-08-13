#![allow(dead_code)]
mod API;
mod api;
mod client;
mod endpoints;
mod useragents;
mod utils;

use std::{error::Error, time::Duration};

use api::{APIs, DataAccount, UsernameBuilder};
use client::Client;
use reqwest::Proxy;

const SESSION_ID: &str =
    "2204721379%3ACe7dYxZd3zOjyB%3A22%3AAYfl8NnG7glYKe3V5fpg2exE7C7yPQTY-Sl-UxBong";
const timeout: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new(timeout, None)?;

    let get_profile = APIs::new(APIs::CurrentUser(String::from(SESSION_ID)));
    let resp = client.execute(&get_profile, None).await?;
    let account = DataAccount::parse(resp.raw(), String::from(SESSION_ID)).unwrap();
    println!("{:?}", account);
    // let edit_profile = APIs::new(APIs::EditProfile(account));

    // let username = UsernameBuilder::new().single("0xhades").build();
    // let resp = client.execute(&edit_profile, Some(&username)).await?;
    // println!("{}", resp.raw());

    Ok(())
}
