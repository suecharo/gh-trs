use anyhow::{anyhow, bail, Result};
use dotenv::dotenv;
use std::env;

#[cfg(not(tarpaulin_include))]
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

#[cfg(not(tarpaulin_include))]
pub fn sapporo_run_dir() -> Result<String> {
    dotenv().ok();
    match env::var("SAPPORO_RUN_DIR") {
        Ok(run_dir) => Ok(run_dir),
        Err(_) => {
            let cwd = env::current_dir()?;
            Ok(cwd
                .join("sapporo_run")
                .to_str()
                .ok_or(anyhow!("Invalid path"))?
                .to_string())
        }
    }
}
