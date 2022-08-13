use rand::{distributions::Alphanumeric, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use regex::Regex;
use reqwest::header::{HeaderMap, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

//FIXME: enhance `is_ok` by spliting the values that is
// compared to the response to avoid change in order, resulting in false negative

use super::endpoints;
use super::useragents::USER_AGENTS;

const DEVICE_VERSION: &str = "135.0.0.34.124";
const BIO: &str = "";

pub struct UsernameBuilder {
    usernames: Vec<String>,
}

impl UsernameBuilder {
    pub fn new() -> Self {
        Self { usernames: vec![] }
    }

    pub fn multi(mut self, usernames: Vec<&str>) -> Self {
        self.usernames.extend(
            usernames
                .clone()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<String>>(),
        );
        self
    }

    pub fn single(mut self, username: &str) -> Self {
        self.usernames.push(username.clone().to_string());
        self
    }

    pub fn build(self) -> Username {
        Username {
            usernames: self.usernames,
        }
    }
}

pub struct Username {
    pub(self) usernames: Vec<String>,
}

impl Username {
    pub fn all(&self) -> Vec<String> {
        self.usernames.clone()
    }
}

fn random_string(rng: &mut ThreadRng, length: usize) -> String {
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}

fn user_agent(version: &str) -> String {
    let dpi = ["480", "320", "640", "515", "120", "160", "240", "800"];
    let manufacturer = [
        "HUAWEI", "Xiaomi", "samsung", "OnePlus", "LGE/lge", "ZTE", "HTC", "LENOVO", "MOTOROLA",
        "NOKIA", "OPPO", "SONY", "VIVO", "LAVA",
    ];

    let mut rng = thread_rng();

    let rand_resolution = rng.gen_range(2..9) * 180;
    let lower_resolution = rand_resolution - 180;

    let android_release = if rng.gen_bool(1.0 / 2.0) {
        format!(
            "{}.{}.{}",
            rng.gen_range(1..7),
            rng.gen_range(0..7),
            rng.gen_range(1..7)
        )
    } else {
        format!("{}.{}", rng.gen_range(1..7), rng.gen_range(0..7))
    };

    format!(
        "Instagram {} Android ({}/{}; {}; {}; {}; {}; {}; {}; en_US)",
        version,
        rng.gen_range(18..25),
        android_release,
        dpi.choose(&mut rng).unwrap_or_else(|| &dpi[0]),
        format!("{}x{}", lower_resolution, rand_resolution),
        manufacturer
            .choose(&mut rng)
            .unwrap_or_else(|| &manufacturer[0]),
        format!(
            "{}-{}",
            manufacturer
                .choose(&mut rng)
                .unwrap_or_else(|| &manufacturer[0]),
            random_string(&mut rng, 5)
        ),
        random_string(&mut rng, 4),
        format!(
            "{}{}",
            random_string(&mut rng, 2),
            rng.gen_range(1000..9999)
        )
    )
}

fn cookies() -> String {
    format!(
        "ig_did={}; ds_user_id={}",
        uuid(),
        random_string(&mut thread_rng(), 15)
    )
}

fn csrftoken() -> String {
    format!("CSRFT-{}", thread_rng().gen_range(0..99999))
}

fn uuid() -> String {
    Uuid::new_v4().to_string().to_uppercase()
}

fn enc_password() -> Result<String, Box<dyn Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(format!("#PWD_INSTAGRAM_BROWSER:0:{}:fpes", timestamp))
}

#[derive(Debug)]
pub struct DataAccount {
    session_id: SessionID,
    email: String,
    phone: String,
    fullname: String,
    fbid: String,
    uid: String,
}

pub enum Method {
    POST,
    GET,
}

pub trait API {
    fn url(&self) -> &str;
    fn data<'a>(&self, username: Option<&'a [String]>) -> HashMap<&'static str, String>;
    fn headers(&self) -> HeaderMap;
    fn is_ok<'a>(
        &self,
        text: &'a str,
        usernames: Option<&'a [String]>,
    ) -> (bool, Option<Vec<String>>);
    fn method(&self) -> Method {
        Method::POST
    }
}

pub type SessionID = String;

pub enum APIs {
    /// **Input**: `1` *username*,
    /// **Output**: `1` *username*
    Create,

    /// **Input**: `1` *username*,
    /// **Output**: `1` *username*
    CreateBusinessValidated,

    /// **Input**: `1` *username*,
    /// **Output**: `1` *username*  
    CreateValidated,

    /// **Input**: `1` *username*,
    /// **Output**: `1` *username*
    CreateBusiness,

    /// **Input**: `3` *username*,
    /// **Output**: `multi` *username*
    WebCreateAjax,

    /// **Input**: `1` *username*,
    /// **Output**: `multi` *username*
    CheckUsername,
    /// **Input**: `3` *username*,
    /// **Output**: `multi` *username*
    UsernameSuggestions,

    /// **Input**: `1` *username*,
    /// **Output**: `0` *username*
    EditProfile(DataAccount),

    /// **Input**: `1` *username*,
    /// **Output**: `0` *username*
    BloksUsernameChange(DataAccount),

    /// **Input**: `0` *username*,
    /// **Output**: `0` *username*
    CurrentUser(SessionID),
}

impl APIs {
    pub fn new(self) -> Box<dyn API> {
        match self {
            APIs::Create => Box::new(Create {}),
            APIs::CreateBusinessValidated => Box::new(CreateBusinessValidated {}),
            APIs::CreateBusiness => Box::new(CreateBusiness {}),
            APIs::CreateValidated => Box::new(CreateValidated {}),
            APIs::WebCreateAjax => Box::new(WebCreateAjax {}),
            APIs::CheckUsername => Box::new(CheckUsername {}),
            APIs::UsernameSuggestions => Box::new(UsernameSuggestions {}),
            APIs::EditProfile(data_account) => Box::new(EditProfile(data_account)),
            APIs::BloksUsernameChange(data_account) => Box::new(BloksUsernameChange(data_account)),
            APIs::CurrentUser(session_id) => Box::new(CurrentUser(session_id)),
        }
    }
}

pub struct Create;
pub struct CreateBusinessValidated;
pub struct CreateValidated;
pub struct CreateBusiness;
pub struct WebCreateAjax;
pub struct CheckUsername;
pub struct UsernameSuggestions;
pub struct CurrentUser(SessionID);
pub struct EditProfile(DataAccount);
pub struct BloksUsernameChange(DataAccount);

impl API for Create {
    fn url(&self) -> &str {
        endpoints::CREATE
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent(DEVICE_VERSION).parse().unwrap());
        headers.insert(COOKIE, cookies().parse().unwrap());
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        const CREATE_AVAILABLE: [&str; 3] = [
            "This password is too easy to guess. Please create a new one",
            "Create a password at least 6 characters long.",
            "{\"account_created\": false, \"errors\": {\"__all__\": [{\"message\": \"Create a password at least 6 characters long.\", \"code\": \"too_short_password\"}]}",
        ];

        (
            CREATE_AVAILABLE
                .iter()
                .map(|x| text.contains(x))
                .filter(|b| *b)
                .collect::<Vec<bool>>()
                .len()
                != 0,
            None,
        )
    }
}

impl API for CreateBusinessValidated {
    fn url(&self) -> &str {
        endpoints::CREATE_BUSINESS_VALIDATED
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent(DEVICE_VERSION).parse().unwrap());
        headers.insert(COOKIE, cookies().parse().unwrap());
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        const CREATE_AVAILABLE: [&str; 3] = [
            "This password is too easy to guess. Please create a new one",
            "Create a password at least 6 characters long.",
            "{\"account_created\": false, \"errors\": {\"__all__\": [{\"message\": \"Create a password at least 6 characters long.\", \"code\": \"too_short_password\"}]}",
        ];

        (
            CREATE_AVAILABLE
                .iter()
                .map(|x| text.contains(x))
                .filter(|b| *b)
                .collect::<Vec<bool>>()
                .len()
                != 0,
            None,
        )
    }
}

impl API for CreateBusiness {
    fn url(&self) -> &str {
        endpoints::CREATE_BUSINESS
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent(DEVICE_VERSION).parse().unwrap());
        headers.insert(COOKIE, cookies().parse().unwrap());
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        const CREATE_AVAILABLE: [&str; 3] = [
            "This password is too easy to guess. Please create a new one",
            "Create a password at least 6 characters long.",
            "{\"account_created\": false, \"errors\": {\"__all__\": [{\"message\": \"Create a password at least 6 characters long.\", \"code\": \"too_short_password\"}]}",
        ];

        (
            CREATE_AVAILABLE
                .iter()
                .map(|x| text.contains(x))
                .filter(|b| *b)
                .collect::<Vec<bool>>()
                .len()
                != 0,
            None,
        )
    }
}

impl API for CreateValidated {
    fn url(&self) -> &str {
        endpoints::CREATE_VALIDATED
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent(DEVICE_VERSION).parse().unwrap());
        headers.insert(COOKIE, cookies().parse().unwrap());
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        const CREATE_AVAILABLE: [&str; 3] = [
            "This password is too easy to guess. Please create a new one",
            "Create a password at least 6 characters long.",
            "{\"account_created\": false, \"errors\": {\"__all__\": [{\"message\": \"Create a password at least 6 characters long.\", \"code\": \"too_short_password\"}]}",
        ];

        (
            CREATE_AVAILABLE
                .iter()
                .map(|x| text.contains(x))
                .filter(|b| *b)
                .collect::<Vec<bool>>()
                .len()
                != 0,
            None,
        )
    }
}

impl API for WebCreateAjax {
    fn url(&self) -> &str {
        endpoints::WEB_CREATE_AJAX
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();

        forms.insert(
            "email",
            format!("{}@gmail.com", usernames.unwrap().get(2).unwrap().clone()),
        );
        forms.insert(
            "first_name",
            usernames.unwrap().get(1).unwrap().clone().to_string(),
        );
        forms.insert("client_id", uuid());
        forms.insert("uuid", uuid());
        forms.insert("seamless_login_enabled", "1".to_string());
        forms.insert("opt_into_one_tap", "false".to_string());
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            USER_AGENTS
                .choose(&mut thread_rng())
                .unwrap_or_else(|| &USER_AGENTS[0])
                .parse()
                .unwrap(),
        );
        headers.insert(
            COOKIE,
            {
                format!(
                    "mid={}; ig_did={}; ig_nrcb=1; csrftoken=missing",
                    "YjDlaAALAAEPz9esG0WWO6Hv4qLb",
                    uuid()
                )
            }
            .parse()
            .unwrap(),
        );
        headers.insert("X-CSRFToken", "missing".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "en;q=0.9".parse().unwrap());
        headers
    }
    fn is_ok<'a>(
        &self,
        text: &'a str,
        usernames: Option<&'a [String]>,
    ) -> (bool, Option<Vec<String>>) {
        if text.contains("{\"account_created\": false, \"errors\": {\"__all__\": [{\"message\": \"Create a password at least 6 characters long.\", \"code\": \"too_short_password\"}]}") {
            return (true, Some(vec![usernames.unwrap().get(0).unwrap().clone().to_string()]));
        }

        if text.contains("username_suggestions") {
            let resp: serde_json::Value = match serde_json::from_str(text) {
                Ok(t) => t,
                Err(_) => return (false, None),
            };

            let username_suggestions = match resp["username_suggestions"].as_array() {
                Some(t) => t,
                _ => return (false, None),
            };

            let usernames = username_suggestions
                .iter()
                .map(|u| u.as_str().unwrap_or_else(|| "error"))
                .filter(|u| u.len() <= 4)
                .map(|u| u.to_string())
                .collect::<Vec<String>>();

            if usernames.len() != 0 {
                return (true, Some(usernames));
            }
        }

        (false, None)
    }
}

impl API for CheckUsername {
    fn url(&self) -> &str {
        endpoints::CHECK_USERNAME
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent("187.0.0.32.120").parse().unwrap());
        headers.insert(COOKIE, cookies().parse().unwrap());
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        if text.contains("This username isn't available") || text.contains("username_") {
            return (false, None);
        }

        if text.contains("suggestions") {
            let resp: serde_json::Value = match serde_json::from_str(text) {
                Ok(t) => t,
                Err(_) => return (false, None),
            };

            let suggestions = match resp["suggestions"].as_array() {
                Some(t) => t,
                _ => return (false, None),
            };

            let usernames = suggestions
                .iter()
                .map(|u| u.as_str().unwrap_or_else(|| "error"))
                .filter(|u| u.len() <= 4)
                .map(|u| String::from(u))
                .collect::<Vec<String>>();

            if usernames.len() != 0 {
                return (true, Some(usernames));
            }
        }

        (false, None)
    }
}

impl API for UsernameSuggestions {
    fn url(&self) -> &str {
        endpoints::USERNAME_SUGGESTIONS
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();

        forms.insert(
            "name",
            format!(
                "{}+{}",
                usernames.unwrap().get(0).unwrap().clone(),
                usernames.unwrap().get(1).unwrap().clone()
            ),
        );
        forms.insert(
            "email",
            format!("{}@gmail.com", usernames.unwrap().get(2).unwrap().clone()),
        );
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent("187.0.0.32.120").parse().unwrap());
        headers.insert(COOKIE, cookies().parse().unwrap());
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        if text.contains("username") {
            let usernames = match Regex::new("\"username\":\\s*\"(.*?)\"")
                .unwrap()
                .captures(text)
            {
                Some(t) => t,
                None => return (false, None),
            };

            let usernames = usernames
                .iter()
                .enumerate()
                .filter(|(i, u)| *i != 0 && u.is_some())
                .map(|(_, u)| u.unwrap().as_str())
                .filter(|u| u.len() <= 4)
                .map(|u| u.to_string())
                .collect::<Vec<String>>();

            if usernames.len() != 0 {
                return (true, Some(usernames));
            }
        }

        (false, None)
    }
}

impl API for EditProfile {
    fn url(&self) -> &str {
        endpoints::EDIT_PROFILE
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut forms = HashMap::new();

        forms.insert("first_name", self.0.fullname.clone());
        forms.insert("email", self.0.email.clone());
        forms.insert("phone_number", self.0.phone.clone());
        forms.insert("biography", BIO.to_string());
        forms.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        forms.insert("chaining_enabled", "on".to_string());
        forms.insert("external_url", "".to_string());
        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent("187.0.0.32.120").parse().unwrap());
        headers.insert(
            COOKIE,
            format!("sessionid={}", self.0.session_id.clone())
                .parse()
                .unwrap(),
        );
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        if text.contains("is_private") {
            (true, None)
        } else {
            (false, None)
        }
    }
}

impl API for BloksUsernameChange {
    fn url(&self) -> &str {
        endpoints::BLOKS_USERNAME_CHANGE
    }
    fn data<'a>(&self, usernames: Option<&'a [String]>) -> HashMap<&'static str, String> {
        let mut client_input_params = HashMap::new();
        client_input_params.insert(
            "username",
            usernames.unwrap().get(0).unwrap().clone().to_string(),
        );
        client_input_params.insert(
            "family_device_id",
            "37e49636-d272-468d-822c-3205a15dab8c".to_string(),
        );

        let mut server_params = HashMap::new();
        server_params.insert("operation_type", "MUTATE".to_string());
        server_params.insert("identity_ids", self.0.fbid.clone());

        let mut params = HashMap::new();
        params.insert("client_input_params", client_input_params);
        params.insert("server_params", server_params);

        let mut forms = HashMap::new();
        forms.insert("params", serde_json::to_string(&params).unwrap());
        forms.insert("_uuid", uuid());
        forms.insert("bk_client_context", "{\"bloks_version\":\"8dab28e76d3286a104a7f1c9e0c632386603a488cf584c9b49161c2f5182fe07\",\"styles_id\":\"instagram\"}".to_string());
        forms.insert(
            "bloks_versioning_id",
            "8dab28e76d3286a104a7f1c9e0c632386603a488cf584c9b49161c2f5182fe07".to_string(),
        );

        forms
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent("187.0.0.32.120").parse().unwrap());
        headers.insert(
            COOKIE,
            format!("sessionid={}", self.0.session_id.clone())
                .parse()
                .unwrap(),
        );
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        if text.contains("\"mode\":\"d\"")
            || text.contains(format!("\"{}\":\"{}\"", self.0.fbid, self.0.session_id).as_str())
        {
            (true, None)
        } else {
            (false, None)
        }
    }
}

impl API for CurrentUser {
    fn method(&self) -> Method {
        Method::GET
    }
    fn url(&self) -> &str {
        endpoints::CURRENT_USER
    }
    fn data<'a>(&self, _: Option<&'a [String]>) -> HashMap<&'static str, String> {
        unimplemented!()
    }
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent("9.6.0").parse().unwrap());
        headers.insert(
            COOKIE,
            format!("sessionid={}", self.0.clone()).parse().unwrap(),
        );
        headers.insert("X-CSRFTOKEN", csrftoken().parse().unwrap());
        headers
    }
    fn is_ok<'a>(&self, text: &'a str, _: Option<&'a [String]>) -> (bool, Option<Vec<String>>) {
        if text.contains("email") || text.contains("full_name") {
            (true, None)
        } else {
            (false, None)
        }
    }
}

impl DataAccount {
    pub fn parse(raw: &str, session_id: String) -> Option<Self> {
        let resp: serde_json::Value = match serde_json::from_str(raw) {
            Ok(t) => t,
            _ => return None,
        };

        if resp["status"].as_str()? == "ok" {
            return Some(Self {
                session_id,
                email: resp["user"]["email"].as_str()?.to_string(),
                phone: resp["user"]["phone_number"].as_str()?.to_string(),
                fullname: resp["user"]["full_name"].as_str()?.to_string(),
                uid: resp["user"]["pk"].as_u64()?.to_string(),
                fbid: resp["user"]["fbid_v2"].as_u64()?.to_string(),
            });
        }

        None
    }
}
