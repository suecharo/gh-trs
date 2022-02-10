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
                    error!("{} to make-template with error: {}", "Failed".red(), e);
                    exit(1);
                }
            }
        }
        args::Args::Validate {
            config_locations,
            github_token,
            ..
        } => {
            info!("{} validate", "Running".green());
            match validate::validate(config_locations, &github_token) {
                Ok(_) => info!("{} validate", "Success".green()),
                Err(e) => {
                    error!("{} to validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };
        }
        args::Args::Test {
            config_locations,
            github_token,
            wes_location,
            docker_host,
            ..
        } => {
            info!("{} validate", "Running".green());
            let configs = match validate::validate(config_locations, &github_token) {
                Ok(configs) => {
                    info!("{} validate", "Success".green());
                    configs
                }
                Err(e) => {
                    error!("{} to validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };

            info!("{} test", "Running".green());
            match test::test(&configs, &wes_location, &docker_host, false) {
                Ok(()) => info!("{} test", "Success".green()),
                Err(e) => {
                    match wes::stop_wes(&docker_host) {
                        Ok(_) => {}
                        Err(e) => error!("{} to stop WES with error: {}", "Failed".red(), e),
                    }
                    error!("{} to test with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };
        }
        args::Args::Publish {
            config_locations,
            github_token,
            repo,
            branch,
            with_test,
            wes_location,
            docker_host,
            from_trs,
            ..
        } => {
            let config_locations = if from_trs {
                info!("Run in gh-trs from_trs mode");
                info!("TRS endpoint: {}", config_locations[0]);
                match config_io::find_config_loc_recursively_from_trs(&config_locations[0]) {
                    Ok(config_locs) => config_locs,
                    Err(e) => {
                        error!("{} to find config locs with error: {}", "Failed".red(), e);
                        exit(1);
                    }
                }
            } else {
                config_locations
            };

            info!("{} validate", "Running".green());
            let configs = match validate::validate(config_locations, &github_token) {
                Ok(configs) => {
                    info!("{} validate", "Success".green());
                    configs
                }
                Err(e) => {
                    error!("{} to validate with error: {}", "Failed".red(), e);
                    exit(1);
                }
            };

            let verified = if with_test {
                info!("{} test", "Running".green());
                match test::test(&configs, &wes_location, &docker_host, true) {
                    Ok(()) => info!("{} test", "Success".green()),
                    Err(e) => {
                        match wes::stop_wes(&docker_host) {
                            Ok(_) => {}
                            Err(e) => error!("{} to stop WES with error: {}", "Failed".red(), e),
                        }
                        error!("{} to test with error: {}", "Failed".red(), e);
                        exit(1);
                    }
                }
                true
            } else {
                false
            };

            info!("{} publish", "Running".green());
            match publish::publish(&configs, &github_token, &repo, &branch, verified) {
                Ok(()) => info!("{} publish", "Success".green()),
                Err(e) => {
                    error!("{} to publish with error: {}", "Failed".red(), e);
                    exit(1);
                }
            }
        }
    }

    Ok(())
}
