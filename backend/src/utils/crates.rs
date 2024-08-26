use reqwest::{header::USER_AGENT, Client};
use urlencoding::encode;

use super::structures::crates::{Crate, CratesResponse};


pub async fn fetch_crates_list(query: Option<impl ToString>, page: Option<u32>) -> Option<Vec<Crate>> {
    let page = page
        .map(|page| format!("page={page}"))
        .unwrap_or("".to_string());

    let query = query
        .map(|query| format!("q={}", encode(&query.to_string())))
        .unwrap_or("".to_string());

    Client::new()
        .get(format!("https://crates.io/api/v1/crates?{query}&{page}").replace("?&", ""))
        .header(USER_AGENT, "RsGround")
        .send()
        .await
        .ok()?
        .json::<CratesResponse>()
        .await
        .map(|res| res.crates)
        .inspect_err(|err| println!("{err}"))
        .ok()
}
