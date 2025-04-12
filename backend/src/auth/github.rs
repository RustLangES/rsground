use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use serde::Deserialize;

use crate::utils::expect_var;

#[derive(Deserialize, Debug)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

pub fn get_oauth_client() -> BasicClient {
    let client_id = ClientId::new(expect_var!("GITHUB_CLIENT_ID"));
    let client_secret = ClientSecret::new(expect_var!("GITHUB_CLIENT_SECRET"));

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("URL de autorización inválida");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("URL de token inválida");

    let redirect_uri =
        RedirectUrl::new(expect_var!("GITHUB_CALLBACK")).expect("URL de redirección inválida");

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_uri)
}

pub async fn fetch_user(access_token: &str) -> Result<GitHubUser, reqwest::Error> {
    let client = reqwest::Client::new();
    let user_url = "https://api.github.com/user";

    let res = client
        .get(user_url)
        .header("User-Agent", "RustLangEs/rsground")
        .bearer_auth(access_token)
        .send()
        .await?;

    let user: GitHubUser = res.json().await?;
    Ok(user)
}
