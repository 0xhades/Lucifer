use crate::{
    apis::{
        APIs, BloksUsernameChange, CheckUsername, Create, CreateBusiness, CreateBusinessValidated,
        CreateValidated, EarnRequest, EditProfile, Session, UsernameBuilder, UsernameSuggestions,
        WebCreateAjax, API,
    },
    app::Status,
    client::{Client, Response},
    config::{Config, ProxyType},
    runner::AppEvent,
    utils::{load_proxies, load_sessions, load_usernames, save_hunt, save_log, split_list},
};
use futures::{future, stream::FuturesUnordered};
use rand::{seq::SliceRandom, thread_rng};
use reqwest::Proxy;
use std::{
    any::Any,
    error::Error,
    iter::StepBy,
    slice::Iter,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::sync::{Mutex, MutexGuard, Semaphore};

type counter = Arc<AtomicUsize>;
const LOGS_PATH: &str = "error.log";
const HUNTS_PATH: &str = "hunt.log";

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

        let proxies = Arc::new(proxies);

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
            .map(|s| Arc::new(Mutex::new(s.unwrap())))
            .collect::<Vec<Arc<Mutex<EarnRequest>>>>();

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

        self.Transmitter.send(AppEvent::List(sessions.len()));

        let mut list_random = None;
        if DONT_WRAP {
            list_random = Some(list.clone());
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

        if !use_random {
            list_random = None;
        }

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

        let (tx, rx) = mpsc::channel::<String>();

        for list in lists {
            let thread_values = (
                Arc::clone(&self.RS),
                Arc::clone(&self.HuntTotal),
                Arc::clone(&self.MissTotal),
                Arc::clone(&self.TakenTotal),
                Arc::clone(&self.should_quit),
                Arc::clone(&self.ErrorTotal),
                self.Transmitter.clone(),
                Arc::clone(&proxies),
                tx.clone(),
                sessions
                    .iter()
                    .map(|s| Arc::clone(s))
                    .collect::<Vec<Arc<Mutex<EarnRequest>>>>(),
            );
            handles.push(thread::spawn(move || {
                worker(
                    Some(list),
                    None,
                    LIMIT,
                    thread_values.1,
                    thread_values.5,
                    thread_values.2,
                    thread_values.3,
                    thread_values.4,
                    thread_values.6.clone(),
                    CONNECT_TIMEOUT,
                    REQUEST_TIMEOUT,
                    thread_values.7,
                    thread_values.9,
                    thread_values.8,
                )
            }));
        }

        if use_random {
            let list = Arc::new(match list_random {
                Some(list) => list,
                None => {
                    self.Transmitter.send(AppEvent::Log((
                        Status::critical(),
                        "Can't use random".to_string(),
                    )));
                    return Some(
                        "An error occurred while trying to use the random system".to_string(),
                    );
                }
            });
            for _ in 0..extra_threads {
                let thread_values = (
                    Arc::clone(&self.RS),
                    Arc::clone(&self.HuntTotal),
                    Arc::clone(&self.MissTotal),
                    Arc::clone(&self.TakenTotal),
                    Arc::clone(&self.should_quit),
                    Arc::clone(&self.ErrorTotal),
                    self.Transmitter.clone(),
                    Arc::clone(&proxies),
                    tx.clone(),
                    sessions
                        .iter()
                        .map(|s| Arc::clone(s))
                        .collect::<Vec<Arc<Mutex<EarnRequest>>>>(),
                );
                let list = Arc::clone(&list);
                handles.push(thread::spawn(move || {
                    worker(
                        None,
                        Some(list),
                        LIMIT,
                        thread_values.1,
                        thread_values.5,
                        thread_values.2,
                        thread_values.3,
                        thread_values.4,
                        thread_values.6.clone(),
                        CONNECT_TIMEOUT,
                        REQUEST_TIMEOUT,
                        thread_values.7,
                        thread_values.9,
                        thread_values.8,
                    )
                }));
            }
        }

        let mut critical = None;
        drop(tx);

        if let Ok(c) = rx.recv() {
            self.should_quit.store(true, Ordering::Release);
            critical = Some(c);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        critical

        // TODO:
        // 1. split usernames [✔]
        // 2. think of requests switch system [✔]
        //    - random every time [✔]
        //    - switch every period of time
        //    - assign for every proxy a system (future not now)
        //    - percentage for every proxy and create proxy object (future not now)
        // 3. random proxies? [✔]
        // 4. sessions:
        //      are randomly picked objects, if one is used, it won't
        //      be used again, if no session left -> end program [✔]

        /*
        TODO:
            - think of a thread system [✔]
            - implement the thread system [✔]
            - use all of the states [✔]
            - figure how to use progress (app.rs) []
            - figure out how to use SOCKS5, HTTP, HTTPS proxies []
            - importing Proxies, Usernames, SessionIDs []
            - how to use the APIs correctly []
            - think of error handling and retrying methods (spam, block...) []
        */
    }
}

fn worker(
    list: Option<Vec<String>>,
    random: Option<Arc<Vec<String>>>,
    limit: usize,
    huntTotal: Arc<AtomicUsize>,
    errorTotal: Arc<AtomicUsize>,
    missTotal: Arc<AtomicUsize>,
    takenTotal: Arc<AtomicUsize>,
    should_quit: Arc<AtomicBool>,
    transmitter: Sender<AppEvent>,
    connect_timeout: Duration,
    request_timeout: Duration,
    proxies: Arc<Vec<Proxy>>,
    sessions: Vec<Arc<Mutex<EarnRequest>>>,
    critical: Sender<String>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut get_username: Box<dyn FnMut() -> String>;

    if random.is_some() {
        let random = random.unwrap();
        get_username = Box::new(move || random.choose(&mut thread_rng()).unwrap().clone());
    } else if list.is_some() {
        let mut list = list.unwrap().into_iter().cycle();
        get_username = Box::new(move || list.next().unwrap().clone());
    } else {
        transmitter.send(AppEvent::Log((
            Status::critical(),
            "There's no list".to_string(),
        )));
        critical.send(String::from(
            "An error occurred while trying to find an available list",
        ));
        return;
    }

    rt.block_on(async move {
        let semaphore = Arc::new(Semaphore::new(limit));

        loop {
            let permits = Arc::clone(&semaphore);
            let _permit = permits.acquire_owned().await.unwrap();
            let proxies = Arc::clone(&proxies);
            let errorTotal = Arc::clone(&errorTotal);
            let takenTotal = Arc::clone(&takenTotal);
            let huntTotal = Arc::clone(&huntTotal);
            let transmitter = transmitter.clone();

            let username = UsernameBuilder::new().single(&get_username()).build();
            let mut attempts = 1;

            let api: APIs = rand::random();
            let username = match api {
                APIs::Create(_) => UsernameBuilder::new().single(&get_username()).build(),
                APIs::CreateBusinessValidated(_) => {
                    UsernameBuilder::new().single(&get_username()).build()
                }
                APIs::CreateValidated(_) => UsernameBuilder::new().single(&get_username()).build(),
                APIs::CreateBusiness(_) => UsernameBuilder::new().single(&get_username()).build(),
                APIs::WebCreateAjax(_) => {
                    attempts = 3;
                    UsernameBuilder::new()
                        .multi(vec![&get_username(), &get_username(), &get_username()])
                        .build()
                }
                APIs::CheckUsername(_) => UsernameBuilder::new().single(&get_username()).build(),
                APIs::UsernameSuggestions(_) => {
                    attempts = 3;
                    UsernameBuilder::new()
                        .multi(vec![&get_username(), &get_username(), &get_username()])
                        .build()
                }
            };

            let mut valid_session: Option<Arc<Mutex<EarnRequest>>> = None;
            for session in &sessions {
                let session_lock = session.lock().await;
                if session_lock.usability() {
                    valid_session = Some(Arc::clone(session));
                    break;
                }
            }

            if valid_session.is_none() {
                transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "No more sessions".to_string(),
                )));
                critical.send(String::from("All sessions have been consumed"));
                return;
            }

            tokio::spawn(async move {
                let client = match Client::new(
                    connect_timeout,
                    request_timeout,
                    Some(proxies.choose(&mut thread_rng()).unwrap()),
                ) {
                    Ok(c) => c,
                    _ => {
                        errorTotal.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                };

                let result = match api {
                    APIs::Create(request) => client.execute(&request, Some(&username)).await,
                    APIs::CreateBusinessValidated(request) => {
                        client.execute(&request, Some(&username)).await
                    }
                    APIs::CreateValidated(request) => {
                        client.execute(&request, Some(&username)).await
                    }
                    APIs::CreateBusiness(request) => {
                        client.execute(&request, Some(&username)).await
                    }
                    APIs::WebCreateAjax(request) => client.execute(&request, Some(&username)).await,
                    APIs::CheckUsername(request) => client.execute(&request, Some(&username)).await,
                    APIs::UsernameSuggestions(request) => {
                        client.execute(&request, Some(&username)).await
                    }
                };

                if let Ok(resp) = result {
                    if resp.status() {
                        if let Some(usernames) = resp.available() {
                            for username in usernames {
                                let valid_session = valid_session.as_ref().unwrap();
                                let mut session = valid_session.lock().await;
                                let request = session.bloks_username_change();
                                match client
                                    .execute(
                                        request,
                                        Some(&UsernameBuilder::new().single(&username).build()),
                                    )
                                    .await
                                {
                                    Ok(resp) if resp.status() => {
                                        transmitter.send(AppEvent::Hunt(username.to_string()));
                                        session.disable(Some(username));
                                        save_hunt(
                                            HUNTS_PATH,
                                            format!(
                                                "{}={}",
                                                username,
                                                session.session().session_id()
                                            )
                                            .as_str(),
                                        );
                                        huntTotal.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(e) => {
                                        transmitter.send(AppEvent::Miss(username.to_string()));
                                        save_log(
                                            LOGS_PATH,
                                            format!("Missing {}: {}", username, e).as_str(),
                                        )
                                    }
                                    _ => {
                                        transmitter.send(AppEvent::Miss(username.to_string()));
                                    }
                                };
                            }
                        } else {
                            for username in username.all() {
                                let valid_session = valid_session.as_ref().unwrap();
                                let mut session = valid_session.lock().await;
                                let request = session.bloks_username_change();
                                match client
                                    .execute(
                                        request,
                                        Some(&UsernameBuilder::new().single(&username).build()),
                                    )
                                    .await
                                {
                                    Ok(resp) if resp.status() => {
                                        transmitter.send(AppEvent::Hunt(username.to_string()));
                                        session.disable(Some(&username));
                                        save_hunt(
                                            HUNTS_PATH,
                                            format!(
                                                "{}={}",
                                                username,
                                                session.session().session_id()
                                            )
                                            .as_str(),
                                        );
                                        huntTotal.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(e) => {
                                        transmitter.send(AppEvent::Miss(username.to_string()));
                                        save_log(
                                            LOGS_PATH,
                                            format!("Missing {}: {}", username, e).as_str(),
                                        )
                                    }
                                    _ => {
                                        transmitter.send(AppEvent::Miss(username.to_string()));
                                    }
                                };
                            }
                        }
                    } else {
                        takenTotal.fetch_add(attempts, Ordering::Relaxed);
                    }
                } else {
                    errorTotal.fetch_add(1, Ordering::Relaxed);
                }

                drop(_permit);
            });

            if should_quit.load(Ordering::Relaxed) {
                break;
            }
        }
    });
}
