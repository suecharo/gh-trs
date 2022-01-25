use crate::utils;
use anyhow::{bail, ensure, Context as AnyhowContext, Result};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use temp_dir::TempDir;
use which::which;

/// Confirm the existence of the git command.
pub fn confirm_existence_of_git_command() -> Result<()> {
    match which("git") {
        Ok(_) => Ok(()),
        Err(_) => bail!("Could not find git command"),
    }
}

/// Util function for handling spawned git processes
///
/// * `cwd` - The working directory to run the command in
/// * `args` - The arguments to pass to the command
fn exec(
    cwd: impl AsRef<Path>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<Output> {
    let mut command = Command::new("git");
    command.current_dir(cwd).args(args);
    Ok(command
        .output()
        .with_context(|| format!("Failed to execute command [{:?}].", command))?)
}

/// Get the repository URL from the current working directory.
pub fn repo_url() -> Result<String> {
    let cwd = env::current_dir()?;
    let result = exec(&cwd, &["config", "--get", "remote.origin.url"])?;
    ensure!(
        result.status.success(),
        "In to get remote url process, the exit status is non-zero."
    );
    Ok(String::from_utf8(result.stdout)?.trim().to_string())
}

/// Get user name using `git config user.name` from the current working directory.
pub fn user_name() -> Result<String> {
    let cwd = env::current_dir()?;
    let result = exec(cwd, &["config", "--get", "user.name"])?;
    ensure!(
        result.status.success(),
        "In the get user name process, the exit status is non-zero."
    );
    let string_stdout = String::from_utf8(result.stdout)?;
    Ok(string_stdout.trim().to_string())
}

/// Get user email using `git config user.email` from the current working directory.
pub fn user_email() -> Result<String> {
    let cwd = env::current_dir()?;
    let result = exec(cwd, &["config", "--get", "user.email"])?;
    ensure!(
        result.status.success(),
        "In the get user email process, the exit status is non-zero."
    );
    let string_stdout = String::from_utf8(result.stdout)?;
    Ok(string_stdout.trim().to_string())
}

/// Prepare the git repository for working.
/// It is created at temp directory.
/// If the branch is exist at the remote, it is checked out.
/// If the branch is not exist at the remote, it is created as an orphan.
///
/// * `ctx` - The runtime context
pub fn prepare_working_repository(ctx: &utils::Context) -> Result<PathBuf> {
    let temp_dir = TempDir::with_prefix("gh-trs")?;
    let work_dir = temp_dir.path();
    match check_branch_exists(&ctx) {
        Ok(_) => {
            // If the branch is exist at the remote, it is checked out.
            let clone_result = exec(
                &work_dir,
                &[
                    "clone",
                    &ctx.repo_url.to_string(),
                    work_dir.to_str().unwrap(),
                    "--branch",
                    &ctx.branch,
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
                &work_dir,
                &[
                    "clone",
                    &ctx.repo_url.to_string(),
                    work_dir.to_str().unwrap(),
                ],
            )?;
            ensure!(
                clone_result.status.success(),
                "In the git clone process, the exit status is non-zero."
            );
            // Checkout the branch as an orphan.
            let checkout_result = exec(&work_dir, &["checkout", "--orphan", &ctx.branch])?;
            ensure!(
                checkout_result.status.success(),
                "In the git checkout process, the exit status is non-zero."
            );
            // Clean checkout branch.
            clean(&work_dir)?;
        }
    }
    Ok(work_dir.to_path_buf())
}

/// Check the existence of the branch in the remote repository.
///
/// * `ctx` - The runtime context
fn check_branch_exists(ctx: &utils::Context) -> Result<()> {
    let result = exec(
        &env::current_dir()?,
        &[
            "ls-remote",
            "--exit-code",
            &ctx.repo_url.to_string(),
            &ctx.branch,
        ],
    )?;
    ensure!(
        result.status.success(),
        format!(
            "Branch: {} does not exist in the remote repository.",
            &ctx.branch
        )
    );
    Ok(())
}

/// Clean up unversioned files.
/// Called after creating an orphaning branch.
///
/// * `wd` - The working directory
fn clean(wd: impl AsRef<Path>) -> Result<()> {
    let result = exec(&wd, &["rm", "--cached", "-r", "."])?;
    ensure!(
        result.status.success(),
        "In the git rm cached process, the exit status is non-zero."
    );
    let result = exec(&wd, &["clean", "-fdx"])?;
    ensure!(
        result.status.success(),
        "In the git clean process, the exit status is non-zero."
    );
    Ok(())
}
