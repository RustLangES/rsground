use std::env::var;

use urlencoding::encode;

const APP_TOKEN: &str = "GITHUB_APP_TOKEN";
const APP_ID: &str = "GITHUB_APP_ID";

pub fn may_authenticate() -> bool {
    var(APP_TOKEN).is_ok()
    && var(APP_ID).is_ok()
}

pub fn get_auth_link(callback: String) -> Option<String> {
    if var(APP_TOKEN).is_err() {
        return None;
    };

    let Ok(app_id) = var(APP_ID)
    else { return None; };

    Some(format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}",
        encode(&app_id),
        encode(&callback)
    ))
}


