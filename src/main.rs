mod args;
mod config;
mod env;
mod github_api;
mod inspect;
mod logger;
mod make_template;
mod publish;
mod raw_url;
mod remote;
mod validate;

use anyhow::Result;
use colored::Colorize;
use log::{debug, error, info};
use std::process::exit;
use structopt::StructOpt;

fn main() -> Result<()> {
    let args = args::Args::from_args();
    let verbose = match args {
        args::Args::MakeTemplate { verbose, .. } => verbose,
        args::Args::Validate { verbose, .. } => verbose,
        args::Args::Publish { verbose, .. } => verbose,
    };
    logger::init_logger(verbose);

    info!("{} gh-trs", "Start".green());
    debug!("args: {:?}", args);

    match args {
        args::Args::MakeTemplate {
            workflow_location,
            github_token,
            output,
            ..
        } => {
            info!("{} make-template", "Running".green());
            match make_template::make_template(&workflow_location, &github_token, &output) {
                Ok(()) => info!("{} make-template", "Success".green()),
                Err(e) => {
                    error!("{} make-template with error: {}", "Failed".red(), e);
                    exit(1);
                }
            }
        }
        args::Args::Validate {
            config_file,
            github_token,
            ..
        } => {
            info!("{} validate", "Running".green());
            match validate::validate(&config_file, &github_token) {
                Ok(()) => info!("{} validate", "Success".green()),
                Err(e) => {
                    error!("{} validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            }
        }
        args::Args::Publish {
            config_file,
            github_token,
            repo,
            branch,
            ..
        } => {
            info!("{} publish", "Running".green());
            match publish::publish(&config_file, &github_token, &repo, &branch) {
                Ok(()) => info!("{} publish", "Success".green()),
                Err(e) => {
                    error!("{} publish with error: {}", "Failed".red(), e);
                    exit(1);
                }
            }
        }
    }

    Ok(())
}