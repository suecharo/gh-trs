use crate::utils::CommitUser;
use crate::Scheme;

use std::fmt;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, ensure};
use anyhow::{Context, Result};
use regex::Regex;
use url::Url;

/// Util function for handling spawned git processes
pub fn exec(git: &str, cwd: &Path, args: &[&str]) -> Result<Output> {
    let mut command = Command::new(git);
    command
        .current_dir(cwd.to_str().with_context(|| {
            format!(
                "In git exec, the given cwd: {:?} cannot be changed to str.",
                cwd
            )
        })?)
        .args(args);
    Ok(command
        .output()
        .with_context(|| format!("Failed to execute command `{:?}`.", command))?)
}

/// Confirm the existence of the git command,
pub fn confirm_existence_of_git_command(git: &str, cwd: &Path) -> Result<()> {
    let result = exec(git, cwd, &["--help"])?;
    ensure!(
        result.status.success(),
        "In the confirm existence of the git command process, the exit status is non-zero."
    );
    Ok(())
}

/// Get user name using `git config`.
pub fn get_user_name(git: &str, cwd: &Path) -> Result<String> {
    let result = exec(git, cwd, &["config", "--get", "user.name"])?;
    ensure!(
        result.status.success(),
        "In the get use name process, the exit status is non-zero."
    );
    let string_stdout =
        String::from_utf8(result.stdout).context("Failed to change stdout to String.")?;
    Ok(string_stdout.trim().to_string())
}

/// Get user email using `git config`.
pub fn get_user_email(git: &str, cwd: &Path) -> Result<String> {
    let result = exec(git, cwd, &["config", "--get", "user.email"])?;
    ensure!(
        result.status.success(),
        "In the get user email process, the exit status is non-zero."
    );
    let string_stdout =
        String::from_utf8(result.stdout).context("Failed to change stdout to String.")?;
    Ok(string_stdout.trim().to_string())
}

// git default ssh url like: git@github.com:suecharo/gh-trs.git
// original ssh url like: ssh://git@github.com/suecharo/gh-trs.git
#[derive(Debug)]
pub struct RepoUrl {
    pub https: Url,
    pub ssh: Url,
    pub scheme: Scheme,
}

impl RepoUrl {
    pub fn new(repo_url: &str, scheme: &Scheme) -> Result<RepoUrl> {
        let parsed_url = match Url::parse(repo_url) {
            Ok(parsed_url) => parsed_url,
            // for ssh
            Err(_) => {
                let re = Regex::new(r"^git@github\.com:[\w\-]*/[\w\-]*\.git$")
                    .context("Failed to compile regular expression.")?;
                if re.is_match(repo_url) {
                    // like: git@github.com:suecharo/gh-trs.git
                    Url::parse(&repo_url.replace("git@github.com:", "ssh://git@github.com/"))
                        .context("Failed to parse git repository URL.")?
                } else {
                    bail!(format!(
                        "The URL: {} is not a valid git repository URL.",
                        repo_url
                    ))
                }
            }
        };
        let host = parsed_url
            .host_str()
            .context("Failed to get the host of the repository URL.")?;
        ensure!(
            host == "github.com",
            "Failed because the host of the repository URL is not github.com."
        );
        if parsed_url.scheme() == "https" {
            let ssh_url = Url::parse(&format!("ssh://git@github.com{}", parsed_url.path()))
                .context("Failed to parse the ssh URL.")?;
            Ok(RepoUrl {
                https: parsed_url,
                ssh: ssh_url,
                scheme: scheme.clone(),
            })
        } else if parsed_url.scheme() == "ssh" {
            let https_url = Url::parse(&format!("https://github.com{}", parsed_url.path()))
                .context("Failed to parse the https URL.")?;
            Ok(RepoUrl {
                https: https_url,
                ssh: parsed_url,
                scheme: scheme.clone(),
            })
        } else {
            bail!("The schema of the URL must be https or ssh.")
        }
    }
}

impl fmt::Display for RepoUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.scheme {
            Scheme::Https => write!(f, "{}", self.https.as_str()),
            Scheme::Ssh => write!(f, "{}", self.ssh.as_str()),
        }
    }
}

/// Get the repo url from the current git repository.
pub fn get_repo_url(git: &str, cwd: &Path, remote: &str, scheme: &Scheme) -> Result<RepoUrl> {
    let result = exec(
        git,
        cwd,
        &["config", "--get", &format!("remote.{}.url", remote)],
    )?;
    ensure!(
        result.status.success(),
        "In the get remote url process, the exit status is non-zero."
    );
    let str_url = String::from_utf8(result.stdout)
        .context("Failed to change stdout to String.")?
        .trim()
        .to_string();
    Ok(RepoUrl::new(&str_url, scheme)?)
}

/// Clone git repository.
/// 1. clone with branch and depth options
/// 2. clone without branch or depth options
pub fn clone(git: &str, cwd: &Path, repo_url: &RepoUrl, branch: &str, remote: &str) -> Result<()> {
    let result = exec(
        git,
        cwd,
        &[
            "clone",
            &repo_url.to_string(),
            cwd.to_str()
                .ok_or(anyhow!("Failed to change the cwd to str."))?,
            "--branch",
            branch,
            "--single-branch",
            "--origin",
            remote,
            "--depth",
            "1",
        ],
    )?;
    if !result.status.success() {
        // try again without branch or depth options
        let result = exec(
            git,
            cwd,
            &[
                "clone",
                &repo_url.to_string(),
                cwd.to_str()
                    .ok_or(anyhow!("Failed to change the cwd to str."))?,
                "--origin",
                remote,
            ],
        )?;
        ensure!(
            result.status.success(),
            "In the git clone process, the exit status is non-zero."
        );
    }
    Ok(())
}

/// Checkout a branch (create an orphan if it doesn't exist on the remote).
pub fn checkout(git: &str, cwd: &Path, branch: &str, remote: &str) -> Result<()> {
    let result = exec(
        git,
        cwd,
        &[
            "ls-remote",
            "--exit-code",
            ".",
            &format!("{}/{}", remote, branch),
        ],
    )?;
    if result.status.success() {
        // branch exists on remote
        let result = exec(git, cwd, &["checkout", branch])?;
        ensure!(
            result.status.success(),
            "In the git checkout process, the exit status is non-zero."
        );
    } else {
        // branch doesn't exist, create an orphan
        let result = exec(git, cwd, &["checkout", "--orphan", branch])?;
        ensure!(
            result.status.success(),
            "In the git checkout process, the exit status is non-zero."
        );
    }
    Ok(())
}

/// Remove all unversined files.
pub fn rm_cache(git: &str, cwd: &Path) -> Result<()> {
    let result = exec(git, cwd, &["rm", "--cached", "-r", "."])?;
    ensure!(
        result.status.success(),
        "In the git rm cache process, the exit status is non-zero."
    );
    Ok(())
}

/// Clean up unversioned files.
pub fn clean(git: &str, cwd: &Path) -> Result<()> {
    let result = exec(git, cwd, &["clean", "-f", "-d"])?;
    ensure!(
        result.status.success(),
        "In the git clean process, the exit status is non-zero."
    );
    Ok(())
}

/// Add files.
pub fn add(git: &str, cwd: &Path) -> Result<()> {
    let result = exec(git, cwd, &["add", "."])?;
    ensure!(
        result.status.success(),
        "In the git add process, the exit status is non-zero."
    );
    Ok(())
}

/// Set the commit user information for the git repository
pub fn set_commit_user(git: &str, cwd: &Path, commit_user: &CommitUser) -> Result<()> {
    let result = exec(git, cwd, &["config", "user.name", &commit_user.name])?;
    ensure!(
        result.status.success(),
        "In the git set user.name process, the exit status is non-zero."
    );
    let result = exec(git, cwd, &["config", "user.email", &commit_user.email])?;
    ensure!(
        result.status.success(),
        "In the git set user.email process, the exit status is non-zero."
    );
    Ok(())
}

/// Commit (if there are any changes).
pub fn commit(git: &str, cwd: &Path, message: &str) -> Result<()> {
    let result = exec(git, cwd, &["diff-index", "--quiet", "HEAD"])?;
    if !result.status.success() {
        // Commit (if there are any changes).
        let result = exec(git, cwd, &["commit", "-m", message])?;
        ensure!(
            result.status.success(),
            "In the git commit process, the exit status is non-zero."
        )
    } else {
        // No change.
    }
    Ok(())
}

/// Add tag.
pub fn tag(git: &str, cwd: &Path, tag: &Option<String>) -> Result<()> {
    match tag {
        Some(tag) => {
            let result = exec(git, cwd, &["tag", tag])?;
            ensure!(
                result.status.success(),
                "In the git tag process, the exit status is non-zero."
            );
        }
        None => {}
    }
    Ok(())
}

/// Push a branch.
pub fn push(git: &str, cwd: &Path, remote: &str, branch: &str) -> Result<()> {
    let result = exec(git, cwd, &["push", "-f", "--tags", remote, branch])?;
    ensure!(
        result.status.success(),
        "In the git push process, the exit status is non-zero."
    );
    Ok(())
}

pub fn delete_ref(git: &str, cwd: &Path, branch: &str) -> Result<()> {
    let result = exec(
        git,
        cwd,
        &["update-ref", "-d", &format!("refs/heads/{}", branch)],
    )?;
    ensure!(
        result.status.success(),
        "In the git delete-ref process, the exit status is non-zero."
    );
    Ok(())
}
