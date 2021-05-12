use crate::utils::CommitUser;

use std::env;
use std::path::Path;
use std::process;
use std::process::{Command, Output};
use url::Url;

pub fn exec(git: &str, cwd: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(git);
    command.current_dir(cwd.to_str().unwrap()).args(args);
    command.output().unwrap()
}

pub fn confirm_existence_of_git_command(git: &str) {
    let result = exec(git, &env::current_dir().unwrap(), &["--help"]);
    if !result.status.success() {
        eprint!("Failed to confirm the existence of git command.");
        process::exit(1);
    }
}

pub fn get_user_name(git: &str) -> String {
    String::from_utf8(
        exec(
            git,
            &env::current_dir().unwrap(),
            &["config", "--get", "user.name"],
        )
        .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

pub fn get_user_email(git: &str) -> String {
    String::from_utf8(
        exec(
            git,
            &env::current_dir().unwrap(),
            &["config", "--get", "user.email"],
        )
        .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

pub fn get_repo_url(git: &str, remote: &str) -> Url {
    Url::parse(
        String::from_utf8(
            exec(
                git,
                &env::current_dir().unwrap(),
                &["config", "--get", &format!("remote.{}.url", remote)],
            )
            .stdout,
        )
        .unwrap()
        .trim(),
    )
    .unwrap()
}

pub fn clone(git: &str, cwd: &Path, repo_url: &Url, branch: &str, remote: &str) {
    let output = exec(
        git,
        cwd,
        &[
            "clone",
            repo_url.as_str(),
            cwd.to_str().unwrap(),
            "--branch",
            branch,
            "--single-branch",
            "--origin",
            remote,
            "--depth",
            "1",
        ],
    );
    if !output.status.success() {
        // try again without branch or depth options
        let output = exec(
            git,
            cwd,
            &[
                "clone",
                repo_url.as_str(),
                cwd.to_str().unwrap(),
                "--origin",
                remote,
            ],
        );
        if !output.status.success() {
            eprint!("Failed to clone the GitHub repository.");
            process::exit(1);
        }
    }
}

pub fn checkout(git: &str, cwd: &Path, branch: &str, remote: &str) {
    if exec(
        git,
        cwd,
        &[
            "ls-remote",
            "--exit-code",
            ".",
            &format!("{}/{}", remote, branch),
        ],
    )
    .status
    .success()
    {
        // branch exists on remote
        exec(git, cwd, &["checkout", branch]);
    } else {
        // branch doesn't exist, create an orphan
        exec(git, cwd, &["checkout", "--orphan", branch]);
    }
}

pub fn rm_cache(git: &str, cwd: &Path) {
    exec(git, cwd, &["rm", "--cached", "-r", "."]);
}

pub fn clean(git: &str, cwd: &Path) {
    exec(git, cwd, &["clean", "-f", "-d"]);
}

pub fn add(git: &str, cwd: &Path) {
    exec(git, cwd, &["add", "."]);
}

pub fn config_user(git: &str, cwd: &Path, commit_user: &CommitUser) {
    exec(git, cwd, &["config", "user.name", &commit_user.name]);
    exec(git, cwd, &["config", "user.email", &commit_user.email]);
}

pub fn commit(git: &str, cwd: &Path, message: &str) {
    let output = exec(git, cwd, &["diff-index", "--quiet", "HEAD"]);
    if !output.status.success() {
        // Commit (if there are any changes).
        exec(git, cwd, &["commit", "-m", message]);
    }
}

pub fn tag(git: &str, cwd: &Path, tag: &str) {
    exec(git, cwd, &["tag", tag]);
}

pub fn push(git: &str, cwd: &Path, remote: &str, branch: &str) {
    exec(git, cwd, &["push", "--tags", remote, branch]);
}
