use crate::git;
use crate::github;
use crate::utils;
use anyhow::{ensure, Result};
use chrono::Utc;
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    tools: Vec<Tool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    id: String,
    url: Url,
    language_type: String,
    attachments: Option<Vec<Attachment>>,
    testing: Option<Testing>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Attachment {
    target: Option<String>,
    url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
struct Testing {
    attachments: Vec<Attachment>,
}

impl Config {
    /// Load a configuration file.
    /// Validate it using the schema.
    ///
    /// * `ctx` - The runtime context.
    pub fn new(ctx: &utils::Context) -> Result<Self> {
        let content = utils::load_config(ctx)?;
        // Validate config_content here by str -> struct
        let config: Config = serde_yaml::from_str(&content)?;
        // Check that there are no duplicate id's
        let ids = config.extract_tool_ids();
        ensure!(
            utils::check_duplicate(ids),
            "There is a duplicate tool id in the config file."
        );
        Ok(config)
    }

    /// Extract the tool ids from the config file.
    fn extract_tool_ids(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.id.clone()).collect()
    }

    /// Convert the GitHub URL to the GitHub raw-contents URL with latest commit hash.
    ///
    /// * `ctx` - The runtime context
    pub fn convert_latest_commit_hash(&mut self, ctx: &utils::Context) {
        let mut cache = github::CommitHashCache::new();
        self.tools
            .iter_mut()
            .for_each(|tool| tool.convert_github_raw_contents_url(&mut cache, ctx));
    }

    pub fn generate_trs_response(&self, ctx: &utils::Context) -> Result<()> {
        let work_dir = git::prepare_working_repository(&ctx)?;
        let config_version = utils::sha256_digest(serde_yaml::to_string(&self)?);

        // pub fn generate_trs_responses(
        //     opt: &Opt,
        //     repo_url: &RepoUrl,
        //     commit_user: &CommitUser,
        //     wd: impl AsRef<Path>,
        //     config: &Config,
        // ) -> Result<()> {
        //     dump_service_info(&opt, &repo_url, &commit_user, &wd)?;
        //     dump_tools(&opt, &wd, &config)?;
        //     Ok(())
        // }

        // 比較
        // generate
        // とりあえず生成して、add and push
        // 差分とかは git にやらせる

        // trs::generate_trs_responses(&opt, &repo_url, &commit_user, &dest_dir, &config)?;
        // git::add_commit_and_push(&opt, &dest_dir, &commit_user)?;
        // println!("Generating CI settings...");
        // let dest_dir = git::prepare_working_repository(&opt, &repo_url, &default_branch)?;
        unimplemented!()
    }

    pub fn testing(&mut self, _ctx: &utils::Context) -> Result<()> {
        unimplemented!()
    }

    pub fn generate_ci_settings(&self, _ctx: &utils::Context) -> Result<()> {
        unimplemented!()
    }
}

impl Tool {
    /// Convert the GitHub URL to the GitHub raw-contents URL with latest commit hash.
    ///
    /// * `cache` - The cache for commit hashes
    /// * `ctx` - The runtime context
    pub fn convert_github_raw_contents_url(
        &mut self,
        cache: &mut github::CommitHashCache,
        ctx: &utils::Context,
    ) {
        match github::convert_github_raw_contents_url(
            &self.url,
            cache,
            ctx.github_token.as_ref().unwrap(),
        ) {
            Ok(converted_url) => {
                self.url = converted_url;
            }
            Err(_) => {}
        };
        match &self.attachments {
            Some(attachments) => {
                self.attachments = Some(
                    attachments
                        .iter()
                        .map(|attachment| attachment.convert_github_raw_contents_url(cache, ctx))
                        .collect::<Vec<Attachment>>(),
                );
            }
            None => {}
        };
        match &self.testing {
            Some(testing) => {
                self.testing = Some(testing.convert_github_raw_contents_url(cache, ctx));
            }
            None => {}
        };
    }
}

impl Attachment {
    /// Convert the GitHub URL to the GitHub raw-contents URL with latest commit hash.
    ///
    /// * `cache` - The cache for commit hashes
    /// * `ctx` - The runtime context
    fn convert_github_raw_contents_url(
        &self,
        cache: &mut github::CommitHashCache,
        ctx: &utils::Context,
    ) -> Self {
        match github::convert_github_raw_contents_url(
            &self.url,
            cache,
            ctx.github_token.as_ref().unwrap(),
        ) {
            Ok(converted_url) => Self {
                target: self.target.clone(),
                url: converted_url,
            },
            Err(_) => self.clone(),
        }
    }
}

impl Testing {
    /// Convert the GitHub URL to the GitHub raw-contents URL with latest commit hash.
    ///
    /// * `cache` - The cache for commit hashes
    /// * `ctx` - The runtime context
    fn convert_github_raw_contents_url(
        &self,
        cache: &mut github::CommitHashCache,
        ctx: &utils::Context,
    ) -> Self {
        Self {
            attachments: self
                .attachments
                .iter()
                .map(|attachment| attachment.convert_github_raw_contents_url(cache, ctx))
                .collect::<Vec<Attachment>>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceInfo {
    id: String,
    name: String,
    r#type: ServiceType,
    description: String,
    organization: ServiceOrganization,
    contact_url: Url,
    documentation_url: Url,
    created_at: String,
    updated_at: String,
    environment: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceType {
    group: String,
    artifact: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceOrganization {
    name: String,
    url: Url,
}

impl ServiceInfo {
    /// The constructor for `ServiceInfo`.
    ///
    /// * `opt` - Argument Parameters defined at `main.rs`
    /// * `repo_url` - The repository URL.
    /// * `commit_user` - The commit user.
    fn new(ctx: &utils::Context, wd: impl AsRef<Path>) -> Result<Self> {
        let repo_owner = utils::repo_owner(&ctx.repo_url)?;
        let repo_name = utils::repo_name(&ctx.repo_url)?;
        let updated_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let created_at = match Self::load(ctx, wd) {
            Ok(service_info) => service_info.created_at,
            Err(_) => updated_at.clone(),
        };
        Ok(Self {
            id: format!("io.github.{}", &repo_owner),
            name: format!("{}/{}", &repo_owner, &repo_name),
            r#type: ServiceType {
                group: format!("io.github.{}", &repo_owner),
                artifact: "TRS".to_string(),
                version: "gh-trs-1.0.0".to_string(),
            },
            description: "Generated by gh-trs.".to_string(),
            organization: ServiceOrganization {
                name: repo_owner.clone(),
                url: Url::parse(&format!("https://github.com/{}", &repo_owner))?,
            },
            contact_url: Url::parse(&format!("mailto:{}", &ctx.user_email))?,
            documentation_url: Url::parse(&format!(
                "https://{}.github.io/{}",
                &repo_owner, &repo_name
            ))?,
            created_at,
            updated_at,
            environment: "prod".to_string(),
            version: format!("{}", Utc::today().format("%Y%m%d")),
        })
    }

    fn load(ctx: &utils::Context, wd: impl AsRef<Path>) -> Result<Self> {
        let content = utils::load_file(Self::path(ctx, wd))?;
        Ok(serde_json::from_str(&content)?)
    }

    fn path(ctx: &utils::Context, wd: impl AsRef<Path>) -> PathBuf {
        wd.as_ref()
            .join(&ctx.dest)
            .join("service-info/index.json")
            .clean()
    }
}
