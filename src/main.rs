mod git;
mod trs;
mod utils;

use std::path::PathBuf;

use structopt::StructOpt;
use temp_dir::TempDir;

#[derive(StructOpt, Debug)]
#[structopt(name = "gh-trs")]
pub struct Opt {
    /// Path or URL to the gh-trs config file
    #[structopt(default_value = "gh-trs.yml")]
    config_file: String,

    /// GitHub repository URL (default: URL of the git repository you are in)
    #[structopt(long, default_value = "")]
    repo_url: String,

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
    #[structopt(short, long, default_value = "")]
    tag: String,

    /// Commit message
    #[structopt(short, long, default_value = "'Updates by gh-trs.'")]
    message: String,

    /// User name used for git commit (defaults to the git config)
    #[structopt(long, default_value = "")]
    user_name: String,

    /// User email used for git commit (defaults to the git config)
    #[structopt(long, default_value = "")]
    user_email: String,

    /// Path to git executable
    #[structopt(long, default_value = "git")]
    git: String,

    /// Environment the service is running in. Suggested values are prod, test, dev, staging.
    #[structopt(long, default_value = "prod")]
    environment: String,
}

fn main() {
    let opt = Opt::from_args();
    git::confirm_existence_of_git_command(&opt.git);
    let repo_url = utils::resolve_repository_url(&opt.git, &opt.remote, &opt.repo_url);
    let commit_user = utils::resolve_commit_user(&opt.git, &opt.user_name, &opt.user_email);
    let _config = utils::load_config(&opt.config_file);
    let temp_dir = TempDir::new().unwrap();
    let dest_dir = temp_dir.path();
    git::clone(&opt.git, &dest_dir, &repo_url, &opt.branch, &opt.remote);
    git::checkout(&opt.git, &dest_dir, &opt.branch, &opt.remote);
    git::rm_cache(&opt.git, &dest_dir);
    git::clean(&opt.git, &dest_dir);
    trs::generate_rest_api(&opt, &repo_url, &commit_user, &dest_dir);
    git::add(&opt.git, &dest_dir);
    git::config_user(&opt.git, &dest_dir, &commit_user);
    git::commit(&opt.git, &dest_dir, &opt.message);
    if &opt.tag != "" {
        // println!("Tagging");
        git::tag(&opt.git, &dest_dir, &opt.tag);
    }
    git::push(&opt.git, &dest_dir, &opt.remote, &opt.branch);
}
