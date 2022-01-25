mod git;
mod github;
mod trs;
mod utils;
use anyhow::Result;
use human_panic::setup_panic;
use std::path::PathBuf;
use std::process;
use structopt::{clap, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "gh-trs",
    about = "Your own TRS API fully based on GitHub to serve workflow metadata"
)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Args {
    /// Path or URL to the gh-trs config file
    config_file: String,

    /// GitHub repository URL (default: GitHub repository of cwd)
    #[structopt(long)]
    repo_url: Option<String>,

    /// Branch name to publish as GitHub Pages
    #[structopt(short, long, default_value = "gh-pages")]
    branch: String,

    /// Where to place the TRS responses in the GitHub Pages branch
    #[structopt(long, parse(from_os_str), default_value = ".")]
    dest: PathBuf,

    /// User name used for git commit (default: using `git config user.name`)
    #[structopt(long)]
    user_name: Option<String>,

    /// User email used for git commit (default: using `git config user.email`)
    #[structopt(long)]
    user_email: Option<String>,

    /// GitHub token used for authentication
    #[structopt(long)]
    github_token: Option<String>,
}

#[cfg(not(tarpaulin))]
fn run() -> Result<()> {
    git::confirm_existence_of_git_command()?;
    utils::log_info("Loading and verifying input arguments...");
    let arg_opt = Args::from_args();
    let ctx = utils::Context::new(arg_opt)?;
    utils::log_info(&format!("Processing in the context below:\n{}", &ctx));
    utils::log_info("Loading the config file and validating it using schema...");
    let mut config = trs::Config::new(&ctx)?;
    if ctx.can_api_request()? {
        utils::log_info("Converting the config to the latest commit hash...");
        config.convert_latest_commit_hash(&ctx);
    }
    utils::log_info("Generating and publishing the TRS response...");
    config.generate_trs_response(&ctx)?;
    if utils::is_ci_mode()? {
        utils::log_info("Since in GitHub Actions, do testing...");
        config.testing(&ctx)?;
        utils::log_info("Generating the TRS response with testing results...");
        config.generate_trs_response(&ctx)?;
    } else {
        utils::log_info("Since in command line, generating GitHub Actions settings...");
        config.generate_ci_settings(&ctx)?;
    }
    utils::log_info("All processing has been done successfully.");
    // trs api endpoint
    Ok(())
}

/// The main entry point for gh-trs.
#[cfg(not(tarpaulin))]
fn main() {
    setup_panic!();
    let result = run();
    match result {
        Ok(_) => {
            process::exit(0);
        }
        Err(e) => {
            utils::log_error(e);
            process::exit(1);
        }
    }
}
