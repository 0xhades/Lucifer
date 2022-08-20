#[cfg(test)]

mod test {
    use std::time::Duration;

    use crate::apis::{
        BloksUsernameChange, CheckUsername, Create, CreateBusinessValidated, CurrentUser,
        DataAccount, Session, UsernameBuilder, UsernameSuggestions, WebCreateAjax,
    };
    use crate::client::Client;
    use crate::utils::split_list;

    use tokio::{
        fs::OpenOptions,
        io::{AsyncReadExt, AsyncWriteExt},
    };

    const SESSION_ID: &str = "54244227166%3Aa2jPBDB8NJZVnn%3A24";
    const TIMEOUT: Duration = Duration::from_secs(10);

    #[test]
    fn split_list_to_parts() {
        println!();

        let list = (1..=100).map(|i| i.to_string()).collect::<Vec<String>>();
        let list = split_list(list, 16, false).unwrap();

        for (i, it) in list.0.into_iter().enumerate() {
            println!("{}: {:?}", i, it.into_iter().collect::<Vec<String>>());
        }

        if list.2 {
            for i in 0..list.1 {
                println!("{}: random (extra threads: {})", i, list.1);
            }
        }
    }

    #[ignore]
    #[tokio::test]
    async fn create_file() {
        let mut Opener = OpenOptions::new();

        let mut file = Opener
            .create(true)
            .read(true)
            .write(true)
            .open("c:\\Users\\0xhades\\Desktop\\config.json")
            .await
            .unwrap();

        file.write_all(b"hello world").await.unwrap();

        let mut text = String::new();
        file.read_to_string(&mut text).await.unwrap();

        println!("{}", text);
    }

    #[ignore]
    #[tokio::test]
    async fn get_profile() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();
        let get_profile = CurrentUser(String::from(SESSION_ID));

        let resp = client.execute(&get_profile, None).await.unwrap();
        let account = DataAccount::parse(resp.raw(), SESSION_ID).unwrap();

        println!("{:?}", account);
    }

    #[ignore]
    #[tokio::test]
    async fn change_username_async() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();

        let account = Session::new(SESSION_ID.to_string(), TIMEOUT, TIMEOUT)
            .await
            .unwrap();
        println!("{:?}", account);

        let bloksUsernameChange = BloksUsernameChange::new(account.information().clone());
        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client
            .execute(&bloksUsernameChange, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn check_username() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();
        let check_username = CheckUsername::new();

        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client
            .execute(&check_username, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn create() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();
        let create = Create::new();

        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client.execute(&create, Some(&username)).await.unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn create_business_validated() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();
        let CreateBusinessValidated = CreateBusinessValidated::new();

        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client
            .execute(&CreateBusinessValidated, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn web_create_ajax() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();
        let WebCreateAjax = WebCreateAjax::new();

        let username = UsernameBuilder::new()
            .multi(vec!["0xhadesssapfpes", "xbisony5fpes", "fpesqwilucifier"])
            .build();
        let resp = client
            .execute(&WebCreateAjax, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn Username_suggestions() {
        let client = Client::new(TIMEOUT, TIMEOUT, None).unwrap();
        let UsernameSuggestions = UsernameSuggestions::new();

        let username = UsernameBuilder::new()
            .multi(vec!["0xhadesssapfpes", "xbisony5fpes", "fpesqwilucifier"])
            .build();
        let resp = client
            .execute(&UsernameSuggestions, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }
}
