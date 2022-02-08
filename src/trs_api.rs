use crate::trs;

use anyhow::{ensure, Result};
use reqwest;
use url::Url;

fn get_request(url: &Url) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url.as_str())
        .header(reqwest::header::ACCEPT, "application/json")
        .send()?;
    let status = response.status();
    ensure!(
        status.is_success(),
        "Failed to get request to {} with status: {}",
        url,
        status
    );
    let body = response.text()?;
    Ok(body)
}

/// /service-info -> trs::ServiceInfo
pub fn get_service_info(owner: impl AsRef<str>, name: impl AsRef<str>) -> Result<trs::ServiceInfo> {
    let url = Url::parse(&format!(
        "https://{}.github.io/{}/service-info",
        owner.as_ref(),
        name.as_ref()
    ))?;
    let body = get_request(&url)?;
    let service_info: trs::ServiceInfo = serde_json::from_str(&body)?;
    Ok(service_info)
}

/// /toolClasses -> trs::ToolClass[]
pub fn get_tool_classes(
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Vec<trs::ToolClass>> {
    let url = Url::parse(&format!(
        "https://{}.github.io/{}/toolClasses",
        owner.as_ref(),
        name.as_ref()
    ))?;
    let body = get_request(&url)?;
    let tool_classes: Vec<trs::ToolClass> = serde_json::from_str(&body)?;
    Ok(tool_classes)
}

/// /tools -> trs::Tool[]
pub fn get_tools(owner: impl AsRef<str>, name: impl AsRef<str>) -> Result<Vec<trs::Tool>> {
    let url = Url::parse(&format!(
        "https://{}.github.io/{}/tools",
        owner.as_ref(),
        name.as_ref()
    ))?;
    let body = get_request(&url)?;
    let tools: Vec<trs::Tool> = serde_json::from_str(&body)?;
    Ok(tools)
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_get_request() -> Result<()> {
        let url = Url::parse("https://suecharo.github.io/gh-pages-rest-api-hosting/foo")?;
        get_request(&url)?;
        Ok(())
    }

    #[test]
    fn test_get_request_not_found() -> Result<()> {
        let url = Url::parse("https://suecharo.github.io/gh-trs/invalid_path")?;
        let res = get_request(&url);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("404"));
        Ok(())
    }
}
