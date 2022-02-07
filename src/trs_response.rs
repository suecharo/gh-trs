use crate::config;
use crate::trs;
use crate::trs_api;

use anyhow::Result;
use serde_json;

pub struct TrsResponse {
    pub service_info: trs::ServiceInfo,
    pub tool_classes: Vec<trs::ToolClass>,
    pub tools: Vec<trs::Tool>,
    pub tools_id: trs::Tool,
    pub tools_id_versions: Vec<trs::ToolVersion>,
    pub tools_id_versions_version: trs::ToolVersion,
    pub tools_id_versions_version_descriptor: trs::FileWrapper,
    pub tools_id_versions_version_files: Vec<trs::ToolFile>,
    pub tools_id_versions_version_tests: Vec<trs::FileWrapper>,
    pub tools_id_versions_version_containerfile: Vec<trs::FileWrapper>,
}

impl TrsResponse {
    pub fn new(
        config: &config::Config,
        owner: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> Result<Self> {
        let service_info = trs::ServiceInfo::new_or_update(
            trs_api::get_service_info(&owner, &name).ok(),
            &config,
            &owner,
            &name,
        )?;
        let tool_classes = generate_tool_classes(owner.as_ref(), name.as_ref())?;

        let mut tools = trs_api::get_tools(&owner, &name)?;
        let tools_id = match tools.iter().find(|t| t.id == config.id) {
            Some(tool) => {
                // update tool
                let mut tool = tool.clone();
                tool.add_new_tool_version(&config, &owner, &name)?;
                tools = tools
                    .into_iter()
                    .filter(|t| t.id != config.id)
                    .collect::<Vec<trs::Tool>>();
                tools.push(tool.clone());
                tool
            }
            None => {
                // create tool and add
                let mut tool = trs::Tool::new(&config, &owner, &name)?;
                tool.add_new_tool_version(&config, &owner, &name)?;
                tools.push(tool.clone());
                tool
            }
        };
        let tools_id_versions = tools_id.versions.clone();
        let tools_id_versions_version = tools_id_versions
            .iter()
            .find(|tv| tv.version() == config.version)
            .unwrap() // already created
            .clone();

        let tools_id_versions_version_descriptor = generate_descriptor(&config)?;
        let tools_id_versions_version_files = generate_tools_id_versions_version_files(&config)?;
        let tools_id_versions_version_tests = generate_tools_id_versions_version_tests(&config)?;

        Ok(Self {
            service_info,
            tool_classes,
            tools,
            tools_id,
            tools_id_versions,
            tools_id_versions_version,
            tools_id_versions_version_descriptor,
            tools_id_versions_version_files,
            tools_id_versions_version_tests,
            tools_id_versions_version_containerfile: vec![],
        })
    }
}

fn generate_tool_classes(owner: &str, name: &str) -> Result<Vec<trs::ToolClass>> {
    match trs_api::get_tool_classes(&owner, &name) {
        Ok(mut tool_classes) => {
            let has_workflow = tool_classes
                .iter()
                .find(|tc| tc.id == Some("workflow".to_string()));
            if has_workflow.is_none() {
                tool_classes.push(trs::ToolClass::default());
            };
            Ok(tool_classes)
        }
        Err(_) => Ok(vec![trs::ToolClass::default()]),
    }
}

fn generate_descriptor(config: &config::Config) -> Result<trs::FileWrapper> {
    let primary_wf = config
        .workflow
        .files
        .iter()
        .find(|f| f.is_primary())
        .unwrap(); // already validated
    Ok(trs::FileWrapper {
        content: None,
        checksum: None,
        url: Some(primary_wf.url.clone()),
    })
}

fn generate_tools_id_versions_version_files(config: &config::Config) -> Result<Vec<trs::ToolFile>> {
    Ok(config
        .workflow
        .files
        .iter()
        .map(|f| trs::ToolFile {
            path: Some(f.url.clone()),
            file_type: Some(trs::FileType::new_from_file_type(&f.r#type)),
            checksum: None,
        })
        .collect())
}

fn generate_tools_id_versions_version_tests(
    config: &config::Config,
) -> Result<Vec<trs::FileWrapper>> {
    Ok(config
        .workflow
        .testing
        .iter()
        .map(|t| {
            Ok(trs::FileWrapper {
                content: Some(serde_json::to_string(&t)?),
                checksum: None,
                url: None,
            })
        })
        .collect::<Result<Vec<_>>>()?)
}
