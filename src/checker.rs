use crate::{
    apis::{
        BloksUsernameChange, CheckUsername, Create, CreateBusiness, CreateBusinessValidated,
        CreateValidated, EarnRequest, EditProfile, Session, UsernameSuggestions, WebCreateAjax,
    },
    app::Status,
    client::Client,
    config::{Config, ProxyType},
    runner::AppEvent,
    utils::{load_proxies, load_sessions, load_usernames, save_log, split_list},
};
use futures::{future, stream::FuturesUnordered};
use std::{
    error::Error,
    iter::StepBy,
    slice::Iter,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::sync::Semaphore;

type counter = Arc<AtomicUsize>;
const LOGS_PATH: &str = "error.log";

pub struct Checker {
    config: Config,
    TakenTotal: counter,
    ErrorTotal: counter,
    MissTotal: counter,
    HuntTotal: counter,
    RS: counter,
    Transmitter: Sender<AppEvent>,
    should_quit: Arc<AtomicBool>,
}

impl Checker {
    pub fn new(
        config: Config,
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
            HuntTotal: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn init(self) -> Option<String> {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(t) => t,
            Err(e) => return Some(format!("Couldn't create a tokio runtime: {}", e)),
        };

        rt.block_on(self.run())
    }

    pub async fn run(self) -> Option<String> {
        let MAX_WORKERS: usize = match self.config.threads().clone() as usize {
            0 => 10,
            t => t,
        };
        let LIMIT: usize = match self.config.limit_per_thread().clone() as usize {
            0 => 50,
            t => t,
        };

        let CONNECT_TIMEOUT: Duration = match self.config.timeout_connect_proxy().clone() {
            t if t.as_secs() == 0 => Duration::from_secs(10),
            t => t,
        };
        let REQUEST_TIMEOUT: Duration = match self.config.timeout_request().clone() {
            t if t.as_secs() == 0 => Duration::from_secs(10),
            t => t,
        };

        let PROXY_TYPE: ProxyType = self.config.proxy_type();
        let PROXY_PATH: String = self.config.proxy_path();
        let USERNAME_PATH: String = self.config.username_path();
        let SESSIONID_PATH: String = self.config.session_path();

        const DONT_WRAP: bool = false;

        let proxies = match load_proxies(&PROXY_PATH) {
            Ok(list) => list,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Loading proxies".to_string(),
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

        if proxies.len() != 0 {
            self.Transmitter.send(AppEvent::Log((
                Status::success(),
                format!("proxies: {}", proxies.len()),
            )));
        } else {
            self.Transmitter.send(AppEvent::Log((
                Status::critical(),
                "proxies: 0".to_string(),
            )));
            return Some("Proxies are empty, or invalid".to_string());
        }

        let list = match load_usernames(&USERNAME_PATH) {
            Ok(list) => list,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Loading usernames".to_string(),
                )));
                save_log(LOGS_PATH, &e.to_string());
                return Some(e.to_string());
            }
        };

        if list.len() != 0 {
            self.Transmitter.send(AppEvent::Log((
                Status::success(),
                format!("usernames: {}", list.len()),
            )));
        } else {
            self.Transmitter.send(AppEvent::Log((
                Status::critical(),
                "usernames: 0".to_string(),
            )));
            return Some("Usernames are empty, or invalid".to_string());
        }

        let sessions = match load_sessions(&SESSIONID_PATH) {
            Ok(list) => list,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Loading sessions".to_string(),
                )));
                save_log(LOGS_PATH, &e.to_string());
                return Some(e.to_string());
            }
        };

        self.Transmitter.send(AppEvent::Log((
            String::new(),
            "Checking sessions".to_string(),
        )));

        let sessions = future::join_all(
            FuturesUnordered::from_iter(sessions.into_iter())
                .into_iter()
                .map(|s| EarnRequest::new(s, CONNECT_TIMEOUT, REQUEST_TIMEOUT)),
        )
        .await;

        let sessions = sessions
            .into_iter()
            .filter(|r| r.is_ok())
            .map(|s| s.unwrap())
            .collect::<Vec<EarnRequest>>();

        if sessions.len() != 0 {
            self.Transmitter.send(AppEvent::Log((
                Status::success(),
                format!("sessions: {}", sessions.len()),
            )));
        } else {
            self.Transmitter.send(AppEvent::Log((
                Status::critical(),
                "sessions: 0".to_string(),
            )));
            return Some("SessionIDs are empty, or invalid".to_string());
        }

        let (lists, extra_threads, use_random) = match split_list(list, MAX_WORKERS, DONT_WRAP) {
            Some(t) => t,
            None => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Can't split list".to_string(),
                )));
                return Some("An error occurred while spliting the usernames".to_string());
            }
        };

        let create = Create::new();
        let create_business_validated = CreateBusinessValidated::new();
        let create_business = CreateBusiness::new();
        let create_validated = CreateValidated::new();

        let web_create_ajax = WebCreateAjax::new();
        let check_username = CheckUsername::new();
        let username_suggestions = UsernameSuggestions::new();

        let calculate_values = (
            Arc::clone(&self.RS),
            Arc::clone(&self.HuntTotal),
            Arc::clone(&self.MissTotal),
            Arc::clone(&self.TakenTotal),
            Arc::clone(&self.should_quit),
        );
        let mut handles: Vec<JoinHandle<()>> = vec![thread::spawn(move || {
            // calculate requests per seconds
            while !calculate_values.4.load(Ordering::Relaxed) {
                let i = ({ calculate_values.2.load(Ordering::Relaxed).clone() }
                    + { calculate_values.1.load(Ordering::Relaxed).clone() }
                    + { calculate_values.3.load(Ordering::Relaxed).clone() });
                thread::sleep(Duration::from_secs(1));
                let f = ({ calculate_values.2.load(Ordering::Relaxed).clone() }
                    + { calculate_values.1.load(Ordering::Relaxed).clone() }
                    + { calculate_values.3.load(Ordering::Relaxed).clone() });
                calculate_values.0.store(f - i, Ordering::Release);
            }
        })];

        for list in lists {
            let thread_values = (
                Arc::clone(&self.RS),
                Arc::clone(&self.HuntTotal),
                Arc::clone(&self.MissTotal),
                Arc::clone(&self.TakenTotal),
                Arc::clone(&self.should_quit),
                Arc::clone(&self.ErrorTotal),
                self.Transmitter.clone(),
            );
            handles.push(thread::spawn(move || {
                worker(
                    list,
                    LIMIT,
                    thread_values.1,
                    thread_values.5,
                    thread_values.2,
                    thread_values.3,
                    thread_values.4,
                    thread_values.6.clone(),
                )
            }));
        }

        if use_random {
            for _ in 0..extra_threads {
                let thread_values = (
                    Arc::clone(&self.RS),
                    Arc::clone(&self.HuntTotal),
                    Arc::clone(&self.MissTotal),
                    Arc::clone(&self.TakenTotal),
                    Arc::clone(&self.should_quit),
                    Arc::clone(&self.ErrorTotal),
                    self.Transmitter.clone(),
                );
                handles.push(thread::spawn(move || {
                    worker_random(
                        LIMIT,
                        thread_values.1,
                        thread_values.5,
                        thread_values.2,
                        thread_values.3,
                        thread_values.4,
                        thread_values.6.clone(),
                    )
                }));
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // TODO:
        // 1. split usernames [✔]
        // 2. random request? think of requests switch system
        // 3. random proxies?
        // 4. sessions:
        //      are randomly picked objects, if one is used, it won't be used again, if no session left -> end program

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

        None
    }
}

fn worker(
    list: Vec<String>,
    limit: usize,
    huntTotal: Arc<AtomicUsize>,
    errorTotal: Arc<AtomicUsize>,
    missTotal: Arc<AtomicUsize>,
    takenTotal: Arc<AtomicUsize>,
    should_quit: Arc<AtomicBool>,
    transmitter: Sender<AppEvent>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let semaphore = Arc::new(Semaphore::new(limit));

        loop {
            let permits = Arc::clone(&semaphore);
            let _permit = permits.acquire_owned().await.unwrap();

            tokio::spawn(async move {
                drop(_permit);
            });

            if should_quit.load(Ordering::Relaxed) {
                break;
            }
        }
    });
}

fn worker_random(
    limit: usize,
    huntTotal: Arc<AtomicUsize>,
    errorTotal: Arc<AtomicUsize>,
    missTotal: Arc<AtomicUsize>,
    takenTotal: Arc<AtomicUsize>,
    should_quit: Arc<AtomicBool>,
    transmitter: Sender<AppEvent>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let semaphore = Arc::new(Semaphore::new(limit));

        loop {
            let permits = Arc::clone(&semaphore);
            let _permit = permits.acquire_owned().await.unwrap();

            tokio::spawn(async move {
                drop(_permit);
            });

            if should_quit.load(Ordering::Relaxed) {
                break;
            }
        }
    });
}
