#[cfg(test)]

mod test {
    use std::{error::Error, time::Duration};

    use crate::api::{APIs, DataAccount, UsernameBuilder};
    use crate::client::Client;
    use reqwest::Proxy;

    const SESSION_ID: &str = "MISSING";
    const TIMEOUT: Duration = Duration::from_secs(10);

    #[ignore = "sessions aren't available"]
    #[tokio::test]
    async fn get_profile() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let get_profile = APIs::new(APIs::CurrentUser(String::from(SESSION_ID)));

        let resp = client.execute(&get_profile, None).await.unwrap();
        let account = DataAccount::parse(resp.raw(), String::from(SESSION_ID)).unwrap();

        println!("{:?}", account);
    }

    #[ignore = "sessions aren't available"]
    #[tokio::test]
    async fn edit_profile() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let get_profile = APIs::new(APIs::CurrentUser(String::from(SESSION_ID)));

        let resp = client.execute(&get_profile, None).await.unwrap();
        let account = DataAccount::parse(resp.raw(), String::from(SESSION_ID)).unwrap();
        println!("{:?}", account);

        let BloksUsernameChange = APIs::new(APIs::BloksUsernameChange(account));
        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client
            .execute(&BloksUsernameChange, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

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

    #[tokio::test]
    async fn create() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let create = APIs::new(APIs::Create);

        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client.execute(&create, Some(&username)).await.unwrap();

        println!("{}", resp.raw());
    }

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

    #[tokio::test]
    async fn create_secondary_account() {
        let client = Client::new(TIMEOUT, None).unwrap();
        let CreateSecondaryAccount = APIs::new(APIs::CreateSecondaryAccount);

        let username = UsernameBuilder::new().single("0xhadesssapfpes").build();
        let resp = client
            .execute(&CreateSecondaryAccount, Some(&username))
            .await
            .unwrap();

        println!("{}", resp.raw());
    }

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
