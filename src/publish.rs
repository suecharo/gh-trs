use crate::config;
use crate::env;
use crate::github_api;
use crate::trs_response;

use anyhow::{anyhow, Result};
use log::info;

#[cfg(not(tarpaulin_include))]
pub fn publish(
    config: &config::Config,
    gh_token: &Option<impl AsRef<str>>,
    repo: impl AsRef<str>,
    branch: impl AsRef<str>,
    verified: bool,
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
            github_api::create_empty_branch(&gh_token, &owner, &name, branch.as_ref())?;
            info!("Branch: {} created", branch.as_ref());
        }
    }

    let branch_sha = github_api::get_branch_sha(&gh_token, &owner, &name, branch.as_ref())?;
    let latest_commit_sha =
        github_api::get_latest_commit_sha(&gh_token, &owner, &name, branch.as_ref(), None)?;
    let trs_response = trs_response::TrsResponse::new(&config, &owner, &name, verified)?;
    let trs_contents = trs_response.generate_contents()?;
    let new_tree_sha =
        github_api::create_tree(&gh_token, &owner, &name, Some(&branch_sha), trs_contents)?;
    let new_commit_sha = github_api::create_commit(
        &gh_token,
        &owner,
        &name,
        Some(&latest_commit_sha),
        &new_tree_sha,
        format!(
            "Add a workflow {} version {} by gh-trs.",
            config.id, config.version
        ),
    )?;
    github_api::update_ref(&gh_token, &owner, &name, branch.as_ref(), &new_commit_sha)?;

    info!(
        "Published to repo: {}/{} branch: {}",
        &owner,
        &name,
        branch.as_ref()
    );
    info!(
        "You can get like:\n    curl -L https://{}.github.io/{}/tools/{}/versions/{}",
        &owner, &name, config.id, config.version
    );

    Ok(())
}
