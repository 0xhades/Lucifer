use crate::{
    app::Status,
    config::{Config, ProxyType},
    runner::AppEvent,
    utils::{load_proxies, load_sessions, load_usernames, save_log},
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        mpsc::Sender,
        Arc,
    },
    time::Duration,
};

type counter = Arc<AtomicUsize>;
const LOGS_PATH: &str = "error.log";

pub struct Checker {
    config: Arc<Config>,
    TakenTotal: counter,
    ErrorTotal: counter,
    MissTotal: counter,
    RS: counter,
    Transmitter: Sender<AppEvent>,
    should_quit: Arc<AtomicBool>,
}

impl Checker {
    pub fn new(
        config: Arc<Config>,
        TakenTotal: counter,
        ErrorTotal: counter,
        MissTotal: counter,
        RS: counter,
        Transmitter: Sender<AppEvent>,
        should_quit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            config,
            TakenTotal,
            ErrorTotal,
            MissTotal,
            RS,
            Transmitter,
            should_quit,
        }
    }

    /*
    TODO:
        - think of a thread system []
        - implement the thread system []
        - use all of the states []
        - figure how to use progress (app.rs) []
        - figure out how to use SOCKS5, HTTP, HTTPS proxies []
        - importing Proxies, Usernames, SessionIDs []
        - how to use the APIs correctly []
        - think of error handling and retrying methods (spam, block...) []
    */

    pub fn run(self) -> Option<String> {
        let MAX_WORKERS: usize = self.config.threads().clone() as usize;
        let LIMIT: usize = self.config.limit_per_thread().clone() as usize;
        let CONNECT_TIMEOUT: Duration = self.config.timeout_connect_proxy().clone();
        let REQUEST_TIMEOUT: Duration = self.config.timeout_request().clone();

        let PROXY_TYPE: ProxyType = self.config.proxy_type();
        let PROXY_PATH: String = self.config.proxy_path();
        let USERNAME_PATH: String = self.config.username_path();
        let SESSIONID_PATH: String = self.config.session_path();

        let proxies = match load_proxies(&PROXY_PATH) {
            Ok(list) => list,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Loading Proxies".to_string(),
                )));
                save_log(LOGS_PATH, &e.to_string());
                return Some(e.to_string());
            }
        };

        let proxies = proxies
            .into_iter()
            .map(|p| match PROXY_TYPE {
                ProxyType::HTTP => reqwest::Proxy::all(format!("http://{}", p)),
                ProxyType::HTTPS => reqwest::Proxy::all(format!("https://{}", p)),
                ProxyType::SOCKS5 => reqwest::Proxy::all(format!("socks5://{}", p)),
            })
            .filter(|p| p.is_ok())
            .map(|p| p.unwrap())
            .collect::<Vec<reqwest::Proxy>>();

        self.Transmitter.send(AppEvent::Log((
            Status::success(),
            format!("Proxies -> {}", proxies.len()),
        )));

        let list = match load_usernames(&USERNAME_PATH) {
            Ok(list) => list,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Loading Usernames".to_string(),
                )));
                save_log(LOGS_PATH, &e.to_string());
                return Some(e.to_string());
            }
        };

        self.Transmitter.send(AppEvent::Log((
            Status::success(),
            format!("Usernames -> {}", list.len()),
        )));

        let sessions = match load_sessions(&SESSIONID_PATH) {
            Ok(list) => list,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Loading SessionIDs".to_string(),
                )));
                save_log(LOGS_PATH, &e.to_string());
                return Some(e.to_string());
            }
        };

        self.Transmitter.send(AppEvent::Log((
            Status::success(),
            format!("SessionIDs -> {}", sessions.len()),
        )));

        

        None
    }
}

pub fn breakdown() {
    /*
        This algortim gonna make the RAM explode if you'll use it with a very big list
        like 15Gb. So find another way when dealing with big lists...
        This is only for small lists...

        This is one of the ways to make mutexless (for the big list) multi-threading..
        Find onther way in the future.

        There are cases that this algortim might broke and need handling:

        - Workers > list size
            - The extra threads going to process random items from
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

    //f
}
