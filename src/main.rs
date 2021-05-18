mod git;
mod github;
mod trs;
mod utils;

use std::env;
use std::path::PathBuf;

use human_panic::setup_panic;
use structopt::clap::arg_enum;
use structopt::{clap, StructOpt};
use temp_dir::TempDir;

arg_enum! {
    #[derive(Debug, Clone)]
    pub enum Scheme {
        Https,
        Ssh
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "gh-trs",
    about = "your own API fully based on GitHub to serve workflow metadata"
)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    /// Path or URL to the gh-trs config file
    #[structopt(default_value = "gh-trs.yml")]
    config_file: String,

    /// GitHub repository URL (default: URL of the git repository you are in)
    #[structopt(long)]
    repo_url: Option<String>,

    /// Name of the branch you are pushing to
    #[structopt(short, long, default_value = "gh-pages")]
    branch: String,

    /// Target directory within the destination branch (relative to the root)
    #[structopt(long, parse(from_os_str), default_value = ".")]
    dest: PathBuf,

    /// Name of the remote
    #[structopt(short, long, default_value = "origin")]
    remote: String,

    /// Add tag to commit
    #[structopt(short, long)]
    tag: Option<String>,

    /// Commit message
    #[structopt(short, long, default_value = "'Updates by gh-trs.'")]
    message: String,

    /// User name used for git commit (defaults to the git config)
    #[structopt(long)]
    user_name: Option<String>,

    /// User email used for git commit (defaults to the git config)
    #[structopt(long)]
    user_email: Option<String>,

    /// Path to git executable
    #[structopt(long, default_value = "git")]
    git: String,

    /// Environment the service is running in. Suggested values are prod, test, dev, staging.
    #[structopt(long, default_value = "prod")]
    environment: String,

    /// Scheme of the repository URL to use in the directory to clone
    #[structopt(short, long, possible_values = &Scheme::variants(), case_insensitive = true, default_value = "Https")]
    scheme: Scheme,
}

fn main() {
    setup_panic!();
    let opt = Opt::from_args();
    let cwd = env::current_dir().expect("Failed to get cwd in your environment.");
    git::confirm_existence_of_git_command(&opt.git, &cwd)
        .expect("Failed to confirm the existence of the git command.");
    let repo_url =
        utils::resolve_repository_url(&opt.git, &cwd, &opt.remote, &opt.repo_url, &opt.scheme)
            .expect("Failed to resolve repository URL.");
    let commit_user = utils::resolve_commit_user(&opt.git, &cwd, &opt.user_name, &opt.user_email)
        .expect("Failed to resolve commit user.");
    let _config = utils::load_config(&opt.config_file).expect("Failed to load config.");
    let temp_dir = TempDir::new().expect("Failed to create temp dir.");
    let dest_dir = temp_dir.path();

    println!("Cloning {} into {:?}", repo_url, dest_dir);
    git::clone(&opt.git, &dest_dir, &repo_url, &opt.branch, &opt.remote)
        .expect("Failed to clone git repository.");
    println!("Checking out {}/{}", opt.remote, opt.branch);
    git::checkout(&opt.git, &dest_dir, &opt.branch, &opt.remote)
        .expect("Failed to checkout git repository.");
    // TODO option history is true, not to do delete_ref
    git::delete_ref(&opt.git, &dest_dir, &opt.branch).expect("Failed to delete ref.");
    git::rm_cache(&opt.git, &dest_dir).expect("Failed to rm.");
    git::clean(&opt.git, &dest_dir).expect("Failed to clean the git repository.");
    println!("Generating the TRS responses");
    trs::generate_trs_responses(&opt, &repo_url, &commit_user, &dest_dir)
        .expect("Failed to generate the TRS responses.");
    println!("Adding all");
    git::add(&opt.git, &dest_dir).expect("Failed to add.");
    git::set_commit_user(&opt.git, &dest_dir, &commit_user).expect("Failed to set commit user.");
    println!("Committing as {} <{}>", commit_user.name, commit_user.email);
    git::commit(&opt.git, &dest_dir, &opt.message).expect("Failed to commit.");
    if opt.tag.is_some() {
        println!("Tagging");
        match git::tag(&opt.git, &dest_dir, &opt.tag) {
            Err(e) => {
                println!("{:?}", e);
                println!("Tagging failed, continuing");
            }
            _ => {}
        }
    }
    println!("Pushing");
    git::push(&opt.git, &dest_dir, &opt.remote, &opt.branch)
        .expect("Failed to push git repository.");
    println!(
        "Your TRS has been deployed to {}",
        trs::trs_url(&repo_url, &opt.dest).expect("Failed to get the TRS URL.")
    );
    println!(
        "Please check `curl -X GET {}/service-info/`",
        trs::trs_url(&repo_url, &opt.dest).expect("Failed to get the TRS URL.")
    );
    github::convert_github_raw_contents_url("FPP").unwrap();
}
