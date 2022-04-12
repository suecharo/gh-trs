use std::path::PathBuf;
use structopt::{clap, StructOpt};
use url::Url;

#[derive(StructOpt, Debug, PartialEq, Clone)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub enum Args {
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    /// Make a template for the gh-trs configuration file.
    MakeTemplate {
        /// Location of the primary workflow document. (only hosted on GitHub)
        workflow_location: Url,

        /// GitHub Personal Access Token.
        #[structopt(long = "gh-token")]
        github_token: Option<String>,

        /// Path to the output file.
        #[structopt(short, long, parse(from_os_str), default_value = "gh-trs-config.yml")]
        output: PathBuf,

        /// Use commit_hash instead of branch in the generated GitHub raw URL.
        #[structopt(long)]
        use_commit_url: bool,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },

    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    /// Validate the schema and contents of the gh-trs configuration file.
    Validate {
        /// Location of the gh-trs configuration files (local file path or remote URL).
        #[structopt(default_value = "gh-trs-config.yml")]
        config_locations: Vec<String>,

        /// GitHub Personal Access Token.
        #[structopt(long = "gh-token")]
        github_token: Option<String>,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },

    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    /// Test the workflow based on the gh-trs configuration file.
    Test {
        /// Location of the gh-trs configuration files (local file path or remote URL).
        #[structopt(default_value = "gh-trs-config.yml")]
        config_locations: Vec<String>,

        /// GitHub Personal Access Token.
        #[structopt(long = "gh-token")]
        github_token: Option<String>,

        /// Location of the WES where the test will be run.
        /// If not specified, `sapporo-service` will be started.
        #[structopt(short, long)]
        wes_location: Option<Url>,

        /// Location of the docker host.
        #[structopt(short, long, default_value = "unix:///var/run/docker.sock")]
        docker_host: Url,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },

    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    /// Publish the TRS response to GitHub.
    Publish {
        /// Location of the gh-trs configuration files (local file path or remote URL).
        #[structopt(default_value = "gh-trs-config.yml")]
        config_locations: Vec<String>,

        /// GitHub Personal Access Token.
        #[structopt(long = "gh-token")]
        github_token: Option<String>,

        /// GitHub repository to publish the TRS response to. (format: <owner>/<repo>)
        #[structopt(short, long, required = true)]
        repo: String,

        /// GitHub branch to publish the TRS response to.
        #[structopt(short, long, default_value = "gh-pages")]
        branch: String,

        /// Test before publishing.
        #[structopt(long)]
        with_test: bool,

        /// Location of the WES where the test will be run.
        /// If not specified, `sapporo-service` will be started.
        #[structopt(short, long)]
        wes_location: Option<Url>,

        /// Location of the docker host.
        #[structopt(short, long, default_value = "unix:///var/run/docker.sock")]
        docker_host: Url,

        /// Recursively get the gh-trs configuration files from the TRS endpoint and publish them.
        /// This option is used to test and publish all workflows in a CI environment.
        /// If you use this option, specify the TRS endpoint for `config_locations`.
        #[structopt(long)]
        from_trs: bool,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_make_template() -> Result<()> {
        let args = Args::from_iter(&[
            "gh-trs",
            "make-template",
            "https://github.com/suecharo/gh-trs/blob/main/path/to/workflow.yml",
        ]);
        assert_eq!(
            args,
            Args::MakeTemplate {
                workflow_location: Url::parse(
                    "https://github.com/suecharo/gh-trs/blob/main/path/to/workflow.yml"
                )?,
                github_token: None,
                output: PathBuf::from("gh-trs-config.yml"),
                use_commit_url: false,
                verbose: false,
            }
        );
        Ok(())
    }

    #[test]
    fn test_validate() -> Result<()> {
        let args = Args::from_iter(&["gh-trs", "validate", "gh-trs-config.yml"]);
        assert_eq!(
            args,
            Args::Validate {
                config_locations: vec!["gh-trs-config.yml".to_string()],
                github_token: None,
                verbose: false,
            }
        );
        Ok(())
    }

    #[test]
    fn test_test() -> Result<()> {
        let args = Args::from_iter(&["gh-trs", "test", "gh-trs-config.yml"]);
        assert_eq!(
            args,
            Args::Test {
                config_locations: vec!["gh-trs-config.yml".to_string()],
                github_token: None,
                wes_location: None,
                docker_host: Url::parse("unix:///var/run/docker.sock")?,
                verbose: false,
            }
        );
        Ok(())
    }

    #[test]
    fn test_publish() -> Result<()> {
        let args = Args::from_iter(&[
            "gh-trs",
            "publish",
            "gh-trs-config.yml",
            "--repo",
            "suecharo/gh-trs",
        ]);
        assert_eq!(
            args,
            Args::Publish {
                config_locations: vec!["gh-trs-config.yml".to_string()],
                repo: "suecharo/gh-trs".to_string(),
                github_token: None,
                branch: "gh-pages".to_string(),
                with_test: false,
                wes_location: None,
                docker_host: Url::parse("unix:///var/run/docker.sock")?,
                from_trs: false,
                verbose: false,
            }
        );
        Ok(())
    }
}
