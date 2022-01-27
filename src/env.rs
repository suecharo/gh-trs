use anyhow::{bail, Result};
use dotenv::dotenv;
use std::env;

pub fn github_token(arg_token: &Option<impl AsRef<str>>) -> Result<String> {
    dotenv().ok();
    match arg_token {
        Some(token) => Ok(token.as_ref().to_string()),
        None => match env::var("GITHUB_TOKEN") {
            Ok(token) => Ok(token),
            Err(_) => bail!("No GitHub token provided. Please set the GITHUB_TOKEN environment variable or pass the --github-token flag."),
        },
    }
}
