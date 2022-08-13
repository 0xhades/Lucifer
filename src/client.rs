use std::error::Error;
use std::time::Duration;

use super::api;
use reqwest;

pub struct Client {
    inner: reqwest::Client,
}

pub struct Response {
    usernames: Option<Vec<String>>,
    status: bool,
    raw: String,
}

impl Response {
    pub fn new(status: bool, usernames: Option<Vec<String>>, raw: String) -> Self {
        Self {
            usernames,
            status,
            raw,
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

impl Client {
    pub fn new(duration: Duration, proxy: Option<reqwest::Proxy>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            inner: match proxy {
                Some(proxy) => reqwest::ClientBuilder::new().proxy(proxy),
                None => reqwest::ClientBuilder::new().no_proxy(),
            }
            .connect_timeout(duration)
            .timeout(duration)
            .tcp_nodelay(true)
            .pool_idle_timeout(None)
            .pool_max_idle_per_host(1)
            .no_gzip()
            .no_deflate()
            .use_rustls_tls()
            .build()?,
        })
    }

    pub async fn execute(
        &self,
        request: &Box<dyn api::API>,
        usernames: Option<&api::Username>,
    ) -> Result<Response, Box<dyn Error>> {
        let mut users: Option<&[String]> = None;
        let mut _mem_holder: Vec<String> = vec![];
        if let Some(username) = usernames {
            _mem_holder = username.all();
            users = Some(_mem_holder.as_slice());
        }

        let text = match request.method() {
            api::Method::POST => self.inner.post(request.url()).form(&request.data(users)),
            api::Method::GET => self.inner.get(request.url()),
        }
        .header("Connection", "Close")
        .headers(request.headers())
        .send()
        .await?
        .text()
        .await?;

        let result = request.is_ok(&text, users);

        Ok(Response::new(result.0, result.1, text))
    }
}
