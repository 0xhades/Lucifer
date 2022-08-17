use crate::{
    config::{Config, ProxyType},
    runner::AppEvent,
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

    pub fn run(self) -> Option<String> {
        let MAX_WORKERS: usize = self.config.threads().clone() as usize;
        let LIMIT: usize = self.config.limit_per_thread().clone() as usize;
        let CONNECT_TIMEOUT: Duration = self.config.timeout_connect_proxy();
        let REQUEST_TIMEOUT: Duration = self.config.timeout_request();

        let PROXY_TYPE: ProxyType = self.config.proxy_type();
        let PROXY_PATH: String = self.config.proxy_path();
        let USERNAME_PATH: String = self.config.username_path();
        let SESSIONID_PATH: String = self.config.session_path();

        /*
        TODO:
            - think of a thread system []
            - implement the thread system []
            - use all of the states []
            - figure how to use progress (app.rs) []
            - figure out how to use SOCKS5, SOCKS4, HTTP, HTTPS proxies []
            - importing Proxies, Usernames, SessionIDs []
            - how to use the APIs correctly []
            - think of error handling and retrying methods (spam, block...) []
        */

        None
    }
}
