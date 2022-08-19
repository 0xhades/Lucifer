use crate::{
    apis::Session,
    app::Status,
    config::{Config, ProxyType},
    runner::AppEvent,
    utils::{load_proxies, load_sessions, load_usernames, save_log},
};
use futures::future;
use std::{
    iter::StepBy,
    slice::Iter,
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

        self.Transmitter.send(AppEvent::Log((
            String::new(),
            "Checking sessions".to_string(),
        )));

        let sessions = match future::try_join_all(
            sessions
                .into_iter()
                .map(|s| Session::new(s, CONNECT_TIMEOUT, REQUEST_TIMEOUT)),
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                self.Transmitter.send(AppEvent::Log((
                    Status::critical(),
                    "Can't check sessions".to_string(),
                )));
                save_log(LOGS_PATH, &e.to_string());
                return Some(format!(
                    "An error occurred while checking all the session IDs: {}",
                    e
                ));
            }
        };

        // 1. split usernames
        // 2. random proxies?
        // 3. sessions:
        //      are randomly picked objects, if one is used, it won't be used again, if no session left -> end program
        // 4.

        None
    }
}

/// skip the iterator to (n).
/// Warning: Please don't use it with an iterator that used `.next()`,
/// because cannot get the current index of an iterator.
fn advance_by<T>(mut iterator: T, n: usize, length: usize) -> Option<T>
where
    T: Iterator,
{
    if n >= length {
        return None;
    }

    for i in 0..n {
        if let None = iterator.next() {
            return None;
        }
    }

    Some(iterator)
}

/// `dont_wrap:`
/// dont repeat values (wrap) **if threads count is more than the list size** and
/// stick with the full list.
pub fn split_list(
    list: &Vec<String>,
    parts: i32,
    mut dont_wrap: bool,
) -> Option<(Vec<StepBy<Iter<String>>>, i32, bool)> {
    /*
        This algortim gonna make the RAM explode if you'll use it with a very big list
        like 15Gb. So find another way when dealing with big lists...
        This is only for small lists...

        This is one of the ways to make mutexless (for the big list) multi-threading..
        Find onther way in the future.

        There are cases that this algortim might broke and need handling:

        - Workers > list size
            ✔ The extra threads going to process random items from
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

    let LIST_SIZE = list.len() as i32;
    let PARTS = parts;

    let extra_workers_count: i32 = PARTS - LIST_SIZE;

    // force check
    if extra_workers_count >= PARTS {
        dont_wrap = false;
    }

    let mut emergency_n: i32 = -1;
    let worker_iters: Vec<StepBy<Iter<String>>> = (0..PARTS)
        .map(|i| advance_by(list.iter(), i as usize, LIST_SIZE as usize))
        .filter(|i| i.is_some() || !dont_wrap)
        .map(|i| {
            i.unwrap_or_else(|| {
                // handling extra workers
                emergency_n += 1;
                advance_by(list.iter(), emergency_n as usize, LIST_SIZE as usize).unwrap_or_else(
                    || {
                        emergency_n = 0;
                        advance_by(list.iter(), emergency_n as usize, LIST_SIZE as usize).unwrap()
                    },
                )
            })
            .step_by(PARTS as usize)
        })
        .collect();

    Some((worker_iters, extra_workers_count, dont_wrap))
}
