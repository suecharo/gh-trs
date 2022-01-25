use crate::utils::CommitUser;
use crate::Opt;
use anyhow::{anyhow, bail, ensure, Context, Result};
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use temp_dir::TempDir;
use url::Url;
use which::which;

/// Add files.
///
/// * `opt` - Argument options defined at `main.rs`
/// * `cwd` - The current working directory
fn add(opt: &Opt, wd: impl AsRef<Path>) -> Result<()> {
    let result = exec(&wd, &["add", "."])?;
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
    let result = exec(&wd, &["config", "user.name", &commit_user.name])?;
    ensure!(
        result.status.success(),
        "In the git set user.name process, the exit status is non-zero."
    );
    let result = exec(&wd, &["config", "user.email", &commit_user.email])?;
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
    let result = exec(&wd, &["commit", "-m", message.as_ref()])?;
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
            let result = exec(&wd, &["tag", tag.as_ref()])?;
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use structopt::StructOpt;
//     use temp_dir::TempDir;

//     mod exec {
//         use super::*;

//         #[test]
//         fn ok() {
//             let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
//             exec("git", temp_dir.path(), &["--help"]).unwrap();
//         }

//         #[test]
//         fn err() {
//             let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
//             assert!(exec("foobar", temp_dir.path(), &[] as &[&str]).is_err());
//         }
//     }

//     mod confirm_existence_of_git_command {
//         use super::*;

//         #[test]
//         fn ok() {
//             let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml"]);
//             confirm_existence_of_git_command(&opt).unwrap();
//         }

//         #[test]
//         fn err() {
//             let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml", "--git", "foobar"]);
//             assert!(confirm_existence_of_git_command(&opt).is_err());
//         }
//     }

//     mod repo_url {
//         use super::*;

//         #[test]
//         fn ok_https() {
//             let repo_url =
//                 RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
//             assert_eq!(
//                 repo_url.https,
//                 Url::parse("https://github.com/suecharo/gh-trs.git").unwrap()
//             );
//             assert_eq!(
//                 repo_url.ssh,
//                 Url::parse("ssh://git@github.com/suecharo/gh-trs.git").unwrap()
//             );
//             assert_eq!(repo_url.scheme, Scheme::Https);
//         }

//         #[test]
//         fn ok_ssh() {
//             let repo_url =
//                 RepoUrl::new("ssh://git@github.com/suecharo/gh-trs.git", &Scheme::Ssh).unwrap();
//             assert_eq!(
//                 repo_url.https,
//                 Url::parse("https://github.com/suecharo/gh-trs.git").unwrap()
//             );
//             assert_eq!(
//                 repo_url.ssh,
//                 Url::parse("ssh://git@github.com/suecharo/gh-trs.git").unwrap()
//             );
//             assert_eq!(repo_url.scheme, Scheme::Ssh);
//         }

//         #[test]
//         fn ok_github_ssh() {
//             let repo_url =
//                 RepoUrl::new("git@github.com:suecharo/gh-trs.git", &Scheme::Ssh).unwrap();
//             assert_eq!(
//                 repo_url.https,
//                 Url::parse("https://github.com/suecharo/gh-trs.git").unwrap()
//             );
//             assert_eq!(
//                 repo_url.ssh,
//                 Url::parse("ssh://git@github.com/suecharo/gh-trs.git").unwrap()
//             );
//             assert_eq!(repo_url.scheme, Scheme::Ssh);
//         }

//         #[test]
//         fn err() {
//             assert!(RepoUrl::new("https://github.com/suecharo/gh-trs", &Scheme::Https).is_err());
//             assert!(RepoUrl::new(
//                 "https://github.com/suecharo/foobar/gh-trs.git",
//                 &Scheme::Https
//             )
//             .is_err());
//             assert!(
//                 RepoUrl::new("https://example.com/suecharo/gh-trs.git", &Scheme::Https).is_err()
//             );
//             assert!(RepoUrl::new("foobar://example.com", &Scheme::Https).is_err());
//         }

//         #[test]
//         fn display() {
//             let https =
//                 RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
//             println!("{}", https);
//             let ssh = RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Ssh).unwrap();
//             println!("{}", ssh);
//         }
//     }

//     mod repo_url {
//         use super::*;

//         #[test]
//         fn ok() {
//             let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml", "--scheme", "Https"]);
//             let repo_url = repo_url(&opt).unwrap();
//             let ori_repo_url =
//                 RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
//             assert_eq!(repo_url.https, ori_repo_url.https);
//             assert_eq!(repo_url.ssh, ori_repo_url.ssh);
//             assert_eq!(repo_url.scheme, ori_repo_url.scheme);
//         }
//     }
// }
