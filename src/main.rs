mod args;
mod config;
mod config_io;
mod env;
mod github_api;
mod inspect;
mod logger;
mod make_template;
mod publish;
mod raw_url;
mod remote;
mod test;
mod trs;
mod trs_api;
mod trs_response;
mod validate;
mod wes;

use anyhow::Result;
use colored::Colorize;
use log::{debug, error, info};
use std::process::exit;
use structopt::StructOpt;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    let args = args::Args::from_args();
    let verbose = match args {
        args::Args::MakeTemplate { verbose, .. } => verbose,
        args::Args::Validate { verbose, .. } => verbose,
        args::Args::Test { verbose, .. } => verbose,
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
            config_location,
            github_token,
            ..
        } => {
            info!("{} validate", "Running".green());
            match validate::validate(&config_location, &github_token) {
                Ok(_) => info!("{} validate", "Success".green()),
                Err(e) => {
                    error!("{} validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };
        }
        args::Args::Test {
            config_location,
            github_token,
            wes_location,
            docker_host,
            ..
        } => {
            info!("{} validate", "Running".green());
            let config = match validate::validate(&config_location, &github_token) {
                Ok(config) => {
                    info!("{} validate", "Success".green());
                    config
                }
                Err(e) => {
                    error!("{} validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };

            info!("{} test", "Running".green());
            match test::test(&config, &wes_location, &docker_host) {
                Ok(test_results) => match test::check_test_results(&test_results) {
                    Ok(()) => info!("{} test", "Success".green()),
                    Err(e) => {
                        error!("{}", e);
                        exit(1);
                    }
                },
                Err(e) => {
                    match wes::stop_wes(&docker_host) {
                        Ok(_) => {}
                        Err(e) => error!("{} to stop WES with error: {}", "Failed".red(), e),
                    }
                    error!("{} test with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };
        }
        args::Args::Publish {
            config_location,
            github_token,
            repo,
            branch,
            with_test,
            wes_location,
            docker_host,
            ..
        } => {
            info!("{} validate", "Running".green());
            let config = match validate::validate(&config_location, &github_token) {
                Ok(config) => {
                    info!("{} validate", "Success".green());
                    config
                }
                Err(e) => {
                    error!("{} validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };

            let verified = if with_test {
                info!("{} test", "Running".green());
                match test::test(&config, &wes_location, &docker_host) {
                    Ok(test_results) => {
                        match test::check_test_results(&test_results) {
                            Ok(()) => info!("{} test", "Success".green()),
                            Err(e) => error!("{}", e),
                        };
                        true
                    }
                    Err(e) => {
                        match wes::stop_wes(&docker_host) {
                            Ok(_) => {}
                            Err(e) => error!("{} to stop WES with error: {}", "Failed".red(), e),
                        }
                        error!("{} test with error: {}", "Failed".red(), e);
                        exit(1);
                    }
                }
            } else {
                false
            };

            info!("{} publish", "Running".green());
            match publish::publish(&config, &github_token, &repo, &branch, verified) {
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
