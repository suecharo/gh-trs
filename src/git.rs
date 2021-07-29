use crate::utils::CommitUser;
use crate::Opt;
use crate::Scheme;
use anyhow::{anyhow, bail, ensure, Context, Result};
use regex::Regex;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use temp_dir::TempDir;
use url::Url;

/// Util function for handling spawned git processes
///
/// * `git` - The git command to run
/// * `cwd` - The working directory to run the command in
/// * `args` - The arguments to pass to the command
fn exec(
    git: impl AsRef<OsStr>,
    cwd: impl AsRef<Path>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<Output> {
    let mut command = Command::new(git);
    command.current_dir(cwd).args(args);
    Ok(command
        .output()
        .with_context(|| format!("Failed to execute command [{:?}].", command))?)
}

/// Confirm the existence of the git command.
///
/// * `opt` - Argument options defined at `main.rs`
pub fn confirm_existence_of_git_command(opt: &Opt) -> Result<()> {
    let cwd = env::current_dir()?;
    let result = exec(&opt.git, cwd, &["--help"])?;
    ensure!(
        result.status.success(),
        "In the confirm existence of the git command process, the exit status is non-zero."
    );
    Ok(())
}

/// Get user name using `git config user.name` from the current working directory.
///
/// * `opt` - Argument options defined at `main.rs`
pub fn get_user_name(opt: &Opt) -> Result<String> {
    let cwd = env::current_dir()?;
    let result = exec(&opt.git, cwd, &["config", "--get", "user.name"])?;
    ensure!(
        result.status.success(),
        "In the get use name process, the exit status is non-zero."
    );
    let string_stdout = String::from_utf8(result.stdout)?;
    Ok(string_stdout.trim().to_string())
}

/// Get user email using `git config user.email` from the current working directory.
///
/// * `opt` - Argument options defined at `main.rs`
pub fn get_user_email(opt: &Opt) -> Result<String> {
    let cwd = env::current_dir()?;
    let result = exec(&opt.git, cwd, &["config", "--get", "user.email"])?;
    ensure!(
        result.status.success(),
        "In the get user email process, the exit status is non-zero."
    );
    let string_stdout = String::from_utf8(result.stdout)?;
    Ok(string_stdout.trim().to_string())
}

/// Structure for repository URL.
/// For push permission, keep both ssh and https URLs.
/// The expected URL to be entered into the constructor is:
///
/// - https URL like: https://github.com/suecharo/gh-trs.git
/// - ssh URL like: ssh://git@github.com/suecharo/gh-trs.git
/// - ssh (github default) URL like: git@github.com:suecharo/gh-trs.git
#[derive(Debug)]
pub struct RepoUrl {
    https: Url,
    ssh: Url,
    scheme: Scheme,
}

impl RepoUrl {
    /// Parse the url into RepoUrl struct.
    ///
    /// * `repo_url` - The repository URL to parse
    /// * `scheme` - The scheme of the url
    ///
    /// Expected repository URL:
    ///
    /// - https URL like: https://github.com/suecharo/gh-trs.git
    /// - ssh URL like: ssh://git@github.com/suecharo/gh-trs.git
    /// - ssh (github default) URL like: git@github.com:suecharo/gh-trs.git
    pub fn new(repo_url: impl AsRef<str>, scheme: &Scheme) -> Result<Self> {
        let re_https = Regex::new(r"^https://github\.com/[\w]*/[\w\-]*(\.git)?$")?;
        let re_ssh = Regex::new(r"^ssh://git@github\.com/[\w]*/[\w\-]*(\.git)?$")?;
        let re_ssh_github = Regex::new(r"^git@github\.com:[\w]*/[\w\-]*(\.git)?$")?;

        let parsed_url = if re_https.is_match(repo_url.as_ref()) {
            Url::parse(repo_url.as_ref())?
        } else if re_ssh.is_match(repo_url.as_ref()) {
            Url::parse(repo_url.as_ref())?
        } else if re_ssh_github.is_match(repo_url.as_ref()) {
            Url::parse(
                &repo_url
                    .as_ref()
                    .replace("git@github.com:", "ssh://git@github.com/"),
            )?
        } else {
            bail!(
                "The inputted URL: {} is not a valid git repository URL.",
                repo_url.as_ref()
            );
        };

        let path = if parsed_url.path().ends_with(".git") {
            parsed_url.path().to_string()
        } else {
            format!("{}.git", parsed_url.path())
        };
        Ok(Self {
            https: Url::parse(&format!("https://github.com{}", &path))?,
            ssh: Url::parse(&format!("ssh://git@github.com{}", &path))?,
            scheme: scheme.clone(),
        })
    }

    pub fn path_segments(&self) -> Result<Vec<&str>> {
        Ok(self
            .https
            .path_segments()
            .ok_or(anyhow!("Failed to parse path in parsed URL."))?
            .collect::<Vec<&str>>())
    }
}

impl fmt::Display for RepoUrl {
    /// `println!` the RepoUrl.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.scheme {
            Scheme::Https => write!(f, "{}", self.https.as_str()),
            Scheme::Ssh => write!(f, "{}", self.ssh.as_str()),
        }
    }
}

/// Get the repo url from the current git repository from the current working directory.
///
/// * `opt` - Argument options defined at `main.rs`
pub fn get_repo_url(opt: &Opt) -> Result<RepoUrl> {
    let cwd = env::current_dir()?;
    let result = exec(
        &opt.git,
        &cwd,
        &["config", "--get", &format!("remote.{}.url", &opt.remote)],
    )?;
    ensure!(
        result.status.success(),
        "In the get remote url process, the exit status is non-zero."
    );
    Ok(RepoUrl::new(
        String::from_utf8(result.stdout)?.trim(),
        &opt.scheme,
    )?)
}

/// Check the existence of the branch in the remote repository.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `repo_url` - The repository URL to check
/// * `branch` - The branch name to check
fn check_branch_exists(opt: &Opt, repo_url: &RepoUrl, branch: impl AsRef<str>) -> Result<()> {
    let cwd = env::current_dir()?;
    let result = exec(
        &opt.git,
        &cwd,
        &[
            "ls-remote",
            "--exit-code",
            &repo_url.to_string(),
            branch.as_ref(),
        ],
    )?;
    ensure!(
        result.status.success(),
        format!(
            "Branch: {} does not exist in the remote repository.",
            branch.as_ref()
        )
    );
    Ok(())
}

/// Clean up unversioned files.
/// Called after creating an orphaning branch.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `wd` - The working directory
fn clean(opt: &Opt, wd: impl AsRef<Path>) -> Result<()> {
    let result = exec(&opt.git, &wd, &["rm", "--cached", "-r", "."])?;
    ensure!(
        result.status.success(),
        "In the git rm cached process, the exit status is non-zero."
    );
    let result = exec(&opt.git, &wd, &["clean", "-fdx"])?;
    ensure!(
        result.status.success(),
        "In the git clean process, the exit status is non-zero."
    );
    Ok(())
}

/// Prepare the git repository for working.
/// It is created at temp directory.
/// If the branch is exist at the remote, it is checked out.
/// If the branch is not exist at the remote, it is created as an orphan.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `repo_url` - The repository URL to check
/// * `branch` - The branch name
pub fn prepare_working_repository(
    opt: &Opt,
    repo_url: &RepoUrl,
    branch: impl AsRef<str>,
) -> Result<PathBuf> {
    let temp_dir = TempDir::with_prefix("gh-trs")?;
    let dest_dir = temp_dir.path();
    match check_branch_exists(&opt, &repo_url, &branch) {
        Ok(_) => {
            // If the branch is exist at the remote, it is checked out.
            let clone_result = exec(
                &opt.git,
                &dest_dir,
                &[
                    "clone",
                    &repo_url.to_string(),
                    dest_dir
                        .to_str()
                        .ok_or(anyhow!("Failed to change the cwd to str."))?,
                    "--branch",
                    branch.as_ref(),
                    "--single-branch",
                    "--depth",
                    "1",
                ],
            )?;
            ensure!(
                clone_result.status.success(),
                "In the git clone process, the exit status is non-zero."
            );
        }
        Err(_) => {
            // If the branch is not exist at the remote, it is created as an orphan.
            let clone_result = exec(
                &opt.git,
                &dest_dir,
                &[
                    "clone",
                    &repo_url.to_string(),
                    dest_dir
                        .to_str()
                        .ok_or(anyhow!("Failed to change the cwd to str."))?,
                ],
            )?;
            ensure!(
                clone_result.status.success(),
                "In the git clone process, the exit status is non-zero."
            );
            // Checkout the branch as an orphan.
            let checkout_result = exec(
                &opt.git,
                &dest_dir,
                &["checkout", "--orphan", branch.as_ref()],
            )?;
            ensure!(
                checkout_result.status.success(),
                "In the git checkout process, the exit status is non-zero."
            );
            // Clean checkout branch.
            clean(&opt, &dest_dir)?;
        }
    }
    Ok(dest_dir.to_path_buf())
}

/// Add files.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `cwd` - The current working directory
fn add(opt: &Opt, wd: impl AsRef<Path>) -> Result<()> {
    let result = exec(&opt.git, &wd, &["add", "."])?;
    ensure!(
        result.status.success(),
        "In the git add process, the exit status is non-zero."
    );
    Ok(())
}

/// Set the commit user information for the git repository.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `wd` - The working directory
/// * `commit_user` - The commit user information
fn set_commit_user(opt: &Opt, wd: impl AsRef<Path>, commit_user: &CommitUser) -> Result<()> {
    let result = exec(&opt.git, &wd, &["config", "user.name", &commit_user.name])?;
    ensure!(
        result.status.success(),
        "In the git set user.name process, the exit status is non-zero."
    );
    let result = exec(&opt.git, &wd, &["config", "user.email", &commit_user.email])?;
    ensure!(
        result.status.success(),
        "In the git set user.email process, the exit status is non-zero."
    );
    Ok(())
}

/// Commit (if there are any changes).
///
/// * `opt` - Argument options defined at `main.rs`
/// * `wd` - The working directory
/// * `message` - The commit message
fn commit(opt: &Opt, wd: impl AsRef<Path>, message: impl AsRef<str>) -> Result<()> {
    let result = exec(&opt.git, &wd, &["commit", "-m", message.as_ref()])?;
    ensure!(
        result.status.success(),
        "In the git commit process, the exit status is non-zero."
    );
    Ok(())
}

/// Add tag.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `wd` - The working directory
/// * `tag` - The tag name
fn tagging<S: AsRef<str>>(opt: &Opt, wd: impl AsRef<Path>, tag: &Option<S>) -> Result<()> {
    match tag {
        Some(tag) => {
            let result = exec(&opt.git, &wd, &["tag", tag.as_ref()])?;
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
///
/// * `opt` - Argument options defined at `main.rs`
/// * `wd` - The working directory
/// * `branch` - The branch name
#[cfg(not(tarpaulin_include))]
fn push(opt: &Opt, wd: impl AsRef<Path>, branch: impl AsRef<str>) -> Result<()> {
    let result = exec(
        &opt.git,
        &wd,
        &["push", "--tags", "origin", branch.as_ref()],
    )?;
    ensure!(
        result.status.success(),
        "In the git push process, the exit status is non-zero."
    );
    Ok(())
}

/// Push the changes to the remote repository.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `wd` - The working directory
/// * `commit_user` - The commit user information
/// * `branch` - The branch name
/// * `message` - The commit message
/// * `tag` - The tag name
pub fn add_commit_and_push<S: AsRef<str>>(
    opt: &Opt,
    wd: impl AsRef<Path>,
    commit_user: &CommitUser,
    branch: impl AsRef<str>,
    message: impl AsRef<str>,
    tag: &Option<S>,
) -> Result<()> {
    add(&opt, &wd)?;
    set_commit_user(&opt, &wd, &commit_user)?;
    commit(&opt, &wd, &message)?;
    tagging(&opt, &wd, &tag)?;
    push(&opt, &wd, &branch)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use structopt::StructOpt;
    use temp_dir::TempDir;

    mod exec {
        use super::*;

        #[test]
        fn ok() {
            let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
            exec("git", temp_dir.path(), &["--help"]).unwrap();
        }

        #[test]
        fn err() {
            let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
            assert!(exec("foobar", temp_dir.path(), &[] as &[&str]).is_err());
        }
    }

    mod confirm_existence_of_git_command {
        use super::*;

        #[test]
        fn ok() {
            let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml"]);
            confirm_existence_of_git_command(&opt).unwrap();
        }

        #[test]
        fn err() {
            let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml", "--git", "foobar"]);
            assert!(confirm_existence_of_git_command(&opt).is_err());
        }
    }

    mod repo_url {
        use super::*;

        #[test]
        fn ok_https() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert_eq!(
                repo_url.https,
                Url::parse("https://github.com/suecharo/gh-trs.git").unwrap()
            );
            assert_eq!(
                repo_url.ssh,
                Url::parse("ssh://git@github.com/suecharo/gh-trs.git").unwrap()
            );
            assert_eq!(repo_url.scheme, Scheme::Https);
        }

        #[test]
        fn ok_ssh() {
            let repo_url =
                RepoUrl::new("ssh://git@github.com/suecharo/gh-trs.git", &Scheme::Ssh).unwrap();
            assert_eq!(
                repo_url.https,
                Url::parse("https://github.com/suecharo/gh-trs.git").unwrap()
            );
            assert_eq!(
                repo_url.ssh,
                Url::parse("ssh://git@github.com/suecharo/gh-trs.git").unwrap()
            );
            assert_eq!(repo_url.scheme, Scheme::Ssh);
        }

        #[test]
        fn ok_github_ssh() {
            let repo_url =
                RepoUrl::new("git@github.com:suecharo/gh-trs.git", &Scheme::Ssh).unwrap();
            assert_eq!(
                repo_url.https,
                Url::parse("https://github.com/suecharo/gh-trs.git").unwrap()
            );
            assert_eq!(
                repo_url.ssh,
                Url::parse("ssh://git@github.com/suecharo/gh-trs.git").unwrap()
            );
            assert_eq!(repo_url.scheme, Scheme::Ssh);
        }

        #[test]
        fn err() {
            assert!(RepoUrl::new("https://github.com/suecharo/gh-trs", &Scheme::Https).is_err());
            assert!(RepoUrl::new(
                "https://github.com/suecharo/foobar/gh-trs.git",
                &Scheme::Https
            )
            .is_err());
            assert!(
                RepoUrl::new("https://example.com/suecharo/gh-trs.git", &Scheme::Https).is_err()
            );
            assert!(RepoUrl::new("foobar://example.com", &Scheme::Https).is_err());
        }

        #[test]
        fn display() {
            let https =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            println!("{}", https);
            let ssh = RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Ssh).unwrap();
            println!("{}", ssh);
        }
    }

    mod get_repo_url {
        use super::*;

        #[test]
        fn ok() {
            let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml", "--scheme", "Https"]);
            let repo_url = get_repo_url(&opt).unwrap();
            let ori_repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert_eq!(repo_url.https, ori_repo_url.https);
            assert_eq!(repo_url.ssh, ori_repo_url.ssh);
            assert_eq!(repo_url.scheme, ori_repo_url.scheme);
        }
    }
}
