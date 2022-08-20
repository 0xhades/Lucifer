use std::error::Error;
use std::time::Duration;

use super::apis;
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

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn available(&self) -> Option<&Vec<String>> {
        self.usernames.as_ref()
    }
}

impl Client {
    pub fn new(
        connect_timeout: Duration,
        request_timeout: Duration,
        proxy: Option<&reqwest::Proxy>,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            inner: match proxy {
                Some(proxy) => reqwest::ClientBuilder::new().proxy(proxy.clone()),
                None => reqwest::ClientBuilder::new().no_proxy(),
            }
            .connect_timeout(connect_timeout)
            .timeout(request_timeout)
            .tcp_nodelay(true)
            .pool_idle_timeout(None)
            .pool_max_idle_per_host(1)
            .no_gzip()
            .no_deflate()
            .use_rustls_tls()
            .build()?,
        })
    }

    pub async fn execute<T>(
        &self,
        request: &T,
        usernames: Option<&apis::Username>,
    ) -> Result<Response, String>
    where
        T: apis::API,
    {
        let mut users: Option<&[String]> = None;
        let mut _mem_holder: Vec<String> = vec![];
        if let Some(username) = usernames {
            _mem_holder = username.all();
            users = Some(_mem_holder.as_slice());
        }

        let text = match match match request.method() {
            apis::Method::POST => self.inner.post(request.url()).form(&request.data(users)),
            apis::Method::GET => self.inner.get(request.url()),
        }
        .header("Connection", "Close")
        .headers(request.headers())
        .send()
        .await
        {
            Ok(it) => it,
            Err(err) => return Err(err.to_string()),
        }
        .text()
        .await
        {
            Ok(it) => it,
            Err(err) => return Err(err.to_string()),
        };

        let (status, result_usernames) = request.is_ok(&text, users);
        Ok(Response::new(status, result_usernames, text))
    }
}
