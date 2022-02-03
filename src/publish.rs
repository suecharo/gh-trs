use crate::config;
use crate::env;
use crate::github_api;

use anyhow::{anyhow, Result};
use log::{info, warn};
use std::path::Path;

#[cfg(not(tarpaulin_include))]
pub fn publish(
    config: &config::Config,
    gh_token: &Option<impl AsRef<str>>,
    repo: impl AsRef<str>,
    branch: impl AsRef<str>,
) -> Result<()> {
    let gh_token = env::github_token(gh_token)?;

    let (owner, name) = github_api::parse_repo(repo)?;
    github_api::get_repos(&gh_token, &owner, &name)
        .map_err(|e| anyhow!("Failed to get repo: {}/{} caused by: {}", owner, name, e))?;

    info!(
        "Publishing to repo: {}/{} branch: {}",
        &owner,
        &name,
        branch.as_ref(),
    );

    match github_api::exists_branch(&gh_token, &owner, &name, branch.as_ref()) {
        Ok(_) => {}
        Err(_) => {
            info!("Branch: {} does not exist, creating it", branch.as_ref());
            github_api::create_branch(&gh_token, &owner, &name, branch.as_ref())?;
            info!("Branch: {} created", branch.as_ref());
        }
    }

    Ok(())
}
