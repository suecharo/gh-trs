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
        .with_context(|| format!("Failed to execute command [{:?}].", command))?)
}

/// Confirm the existence of the git command.
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
    /// Expected repository URL:
    ///
    /// - https://github.com/suecharo/gh-trs.git
    /// - ssh://git@github.com/suecharo/gh-trs.git
    /// - git@github.com:suecharo/gh-trs.git
    pub fn new(repo_url: &str, scheme: &Scheme) -> Result<RepoUrl> {
        let re_https = Regex::new(r"^https://github\.com/[\w]*/[\w\-]*\.git$")
            .context("Failed to compile regular expression.")?;
        let re_ssh = Regex::new(r"^ssh://git@github\.com/[\w]*/[\w\-]*\.git$")
            .context("Failed to compile regular expression.")?;
        let re_ssh_github = Regex::new(r"^git@github\.com:[\w]*/[\w\-]*\.git$")
            .context("Failed to compile regular expression.")?;

        let parsed_url = if re_https.is_match(repo_url) {
            Url::parse(repo_url)
                .with_context(|| format!("Failed to parse the URL: {}", repo_url))?
        } else if re_ssh.is_match(repo_url) {
            Url::parse(repo_url)
                .with_context(|| format!("Failed to parse the URL: {}", repo_url))?
        } else if re_ssh_github.is_match(repo_url) {
            Url::parse(&repo_url.replace("git@github.com:", "ssh://git@github.com/"))
                .with_context(|| format!("Failed to parse the URL: {}", repo_url))?
        } else {
            bail!(format!(
                "The URL: {} is not a valid git repository URL.",
                repo_url
            ))
        };

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
            unreachable!()
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
pub fn clone(git: &str, cwd: &Path, repo_url: &RepoUrl, branch: &str) -> Result<()> {
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
pub fn checkout(git: &str, cwd: &Path, branch: &str) -> Result<()> {
    let result = exec(
        git,
        cwd,
        &[
            "ls-remote",
            "--exit-code",
            ".",
            &format!("origin/{}", branch),
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

/// Delete ref to remove branch history.
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

/// Set the commit user information for the git repository.
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
pub fn push(git: &str, cwd: &Path, branch: &str) -> Result<()> {
    let result = exec(git, cwd, &["push", "-f", "--tags", "origin", branch])?;
    ensure!(
        result.status.success(),
        "In the git push process, the exit status is non-zero."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;
    use temp_dir::TempDir;

    mod exec {
        use super::*;

        #[test]
        fn ok() {
            assert!(exec("git", &env::current_dir().unwrap(), &["--help"]).is_ok());
        }

        #[test]
        fn err() {
            assert!(exec("foobar", &env::current_dir().unwrap(), &[]).is_err());
            assert!(exec("git", &Path::new("/foobar"), &[]).is_err());
        }
    }

    mod confirm_existence_of_git_command {
        use super::*;

        #[test]
        fn ok() {
            assert!(confirm_existence_of_git_command("git", &env::current_dir().unwrap()).is_ok());
        }

        #[test]
        fn err() {
            assert!(
                confirm_existence_of_git_command("foobar", &env::current_dir().unwrap()).is_err()
            );
        }
    }

    mod repo_url {
        use super::*;

        #[test]
        fn ok_https() {
            let result = RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https);
            assert!(result.is_ok());
            let repo_url = result.unwrap();
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
            let result = RepoUrl::new("ssh://git@github.com/suecharo/gh-trs.git", &Scheme::Ssh);
            assert!(result.is_ok());
            let repo_url = result.unwrap();
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
            let result = RepoUrl::new("git@github.com:suecharo/gh-trs.git", &Scheme::Ssh);
            assert!(result.is_ok());
            let repo_url = result.unwrap();
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
                RepoUrl::new("https://foobar.com/suecharo/gh-trs.git", &Scheme::Https).is_err()
            );
        }
    }

    mod get_repo_url {
        use super::*;

        #[test]
        fn ok() {
            let result = get_repo_url(
                "git",
                &env::current_dir().unwrap(),
                "origin",
                &Scheme::Https,
            );
            assert!(result.is_ok());
            let repo_url = result.unwrap();
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
        fn err() {
            assert!(get_repo_url(
                "git",
                &env::current_dir().unwrap(),
                "foobar",
                &Scheme::Https
            )
            .is_err());
        }
    }

    mod clone {
        use super::*;

        #[test]
        fn ok_branch_exists() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert!(clone("git", &TempDir::new().unwrap().path(), &repo_url, "main").is_ok());
        }

        #[test]
        fn ok_branch_not_exist() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert!(clone("git", &TempDir::new().unwrap().path(), &repo_url, "foobar").is_ok());
        }
    }

    mod checkout {
        use super::*;

        #[test]
        fn ok_branch_exists() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            let dest_dir = TempDir::new().unwrap();
            clone("git", &dest_dir.path(), &repo_url, "main").unwrap();
            assert!(checkout("git", &dest_dir.path(), "main").is_ok());
        }

        #[test]
        fn ok_branch_not_exist() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            let dest_dir = TempDir::new().unwrap();
            clone("git", &dest_dir.path(), &repo_url, "main").unwrap();
            assert!(checkout("git", &dest_dir.path(), "foobar").is_ok());
        }

        #[test]
        fn err() {
            assert!(checkout("git", &Path::new("/tmp"), "main").is_err());
        }
    }

    mod delete_ref {
        use super::*;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            let dest_dir = TempDir::new().unwrap();
            clone("git", &dest_dir.path(), &repo_url, "main").unwrap();
            assert!(delete_ref("git", &dest_dir.path(), "main").is_ok());
        }

        #[test]
        fn err() {
            assert!(delete_ref("git", &Path::new("/tmp"), "main").is_err());
        }
    }

    mod clean {
        use super::*;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            let dest_dir = TempDir::new().unwrap();
            clone("git", &dest_dir.path(), &repo_url, "main").unwrap();
            assert!(clean("git", &dest_dir.path()).is_ok());
        }

        #[test]
        fn err() {
            assert!(clean("git", &Path::new("/tmp")).is_err());
        }
    }

    mod add {
        use super::*;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            let dest_dir = TempDir::new().unwrap();
            clone("git", &dest_dir.path(), &repo_url, "main").unwrap();
            assert!(add("git", &dest_dir.path()).is_ok());
        }

        #[test]
        fn err() {
            assert!(add("git", &Path::new("/tmp")).is_err());
        }
    }

    mod set_commit_user {
        use super::*;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            let dest_dir = TempDir::new().unwrap();
            clone("git", &dest_dir.path(), &repo_url, "main").unwrap();
            let commit_user = CommitUser {
                name: "suecharo".to_string(),
                email: "suehiro619@gmail.com".to_string(),
            };
            assert!(set_commit_user("git", &dest_dir.path(), &commit_user).is_ok());
        }

        #[test]
        fn err() {
            let commit_user = CommitUser {
                name: "suecharo".to_string(),
                email: "suehiro619@gmail.com".to_string(),
            };
            assert!(set_commit_user("git", &Path::new("/tmp"), &commit_user).is_err());
        }
    }
}
