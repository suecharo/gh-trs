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
pub struct Opt {
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

    /// Option for use in CI (not used in command line)
    #[structopt(long)]
    ci: bool,
}

#[cfg(not(tarpaulin))]
fn run() -> Result<()> {
    let opt = Opt::from_args();
    git::confirm_existence_of_git_command()?;
    let repo_url = utils::resolve_repository_url(&opt)?;
    let default_branch = github::default_branch_name(&repo_url)?;

    let commit_user = utils::resolve_commit_user(&opt)?;
    let config = utils::validate_and_convert_config(&utils::load_config(&opt.config_file)?)?;

    if !opt.ci {
        println!("Generating TRS responses...");
        let dest_dir = git::prepare_working_repository(&opt, &repo_url, &opt.branch)?;
        println!("{:?}", dest_dir);
        trs::generate_trs_responses(&opt, &repo_url, &commit_user, &dest_dir, &config)?;
        git::add_commit_and_push(&opt, &dest_dir, &commit_user)?;
        println!("Generating CI settings...");
        let dest_dir = git::prepare_working_repository(&opt, &repo_url, &default_branch)?;
        // [TODO] ci::generate_ci_settings(&opt, &repo_url, &dest_dir, &config)?;
        git::add_commit_and_push(&opt, &dest_dir, &commit_user)?;
    } else {
        unimplemented!("Not implemented yet.");
    }
    println!(
        "Your TRS has been deployed to {}",
        trs::trs_url(&repo_url, &opt.dest)?
    );
    println!(
        "Please check `curl -X GET {}/service-info/`",
        trs::trs_url(&repo_url, &opt.dest)?
    );
    Ok(())
}

#[cfg(not(tarpaulin))]
fn main() {
    setup_panic!();
    let result = run();
    match result {
        Ok(_) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("[Error]: {:?}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_args {
        use super::*;

        #[test]
        fn ok() {
            let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml"]);
            assert_eq!(opt.config_file, "gh-trs.yml");
            assert!(opt.repo_url.ok_or("").is_err());
            assert_eq!(opt.branch, "gh-pages");
            assert_eq!(opt.dest, PathBuf::from("."));
            assert!(opt.user_name.ok_or("").is_err());
            assert!(opt.user_email.ok_or("").is_err());
            assert_eq!(opt.ci, false);
        }
    }
}
