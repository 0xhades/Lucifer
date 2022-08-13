#[cfg(test)]

mod test {
    use std::{error::Error, time::Duration};

    use crate::api::{APIs, DataAccount, UsernameBuilder};
    use crate::client::Client;
    use reqwest::Proxy;

    const SESSION_ID: &str = "54244227166%3Aa2jPBDB8NJZVnn%3A24";
    const TIMEOUT: Duration = Duration::from_secs(10);

    #[ignore]
    #[tokio::test]
    async fn get_profile() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let get_profile = APIs::new(APIs::CurrentUser(String::from(SESSION_ID)));

        let resp = client.execute(&get_profile, None).await.unwrap();
        let account = DataAccount::parse(resp.raw(), SESSION_ID).unwrap();

        println!("{:?}", account);
    }

    //#[ignore]
    #[tokio::test]
    async fn change_username_async() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let get_profile = APIs::new(APIs::CurrentUser(String::from(SESSION_ID)));

        let resp = client.execute(&get_profile, None).await.unwrap();
        let account = DataAccount::parse(resp.raw(), SESSION_ID).unwrap();
        println!("{}", resp.raw());
        println!("{:?}", account);

        let BloksUsernameChange = APIs::new(APIs::BloksUsernameChange(account));
        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client
            .execute(&BloksUsernameChange, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn check_username() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let check_username = APIs::new(APIs::CheckUsername);

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
        let client = Client::new(TIMEOUT, None).unwrap();
        let create = APIs::new(APIs::Create);

        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client.execute(&create, Some(&username)).await.unwrap();

        println!("{}", resp.raw());
    }

    #[ignore]
    #[tokio::test]
    async fn create_business_validated() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let CreateBusinessValidated = APIs::new(APIs::CreateBusinessValidated);

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
        let client = Client::new(TIMEOUT, None).unwrap();
        let WebCreateAjax = APIs::new(APIs::WebCreateAjax);

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
        let client = Client::new(TIMEOUT, None).unwrap();
        let UsernameSuggestions = APIs::new(APIs::UsernameSuggestions);

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
