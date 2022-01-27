use anyhow::{ensure, Result};
use reqwest;
use url::Url;

pub fn fetch_raw_content(remote_loc: &Url) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(remote_loc.as_str())
        .header(reqwest::header::ACCEPT, "plain/text")
        .send()?;
    ensure!(
        response.status().is_success(),
        "Failed to fetch raw content from {} with status code {}",
        remote_loc.as_str(),
        response.status()
    );

    Ok(response.text()?)
}
