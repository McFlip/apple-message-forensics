use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseMetadata {
    pub case: CaseInfo,
    pub analyst: AnalystInfo,
    pub custodian: CustodianInfo,
    pub report: ReportInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseInfo {
    pub name: String,
    pub number: String,
    pub requestor: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalystInfo {
    pub name: String,
    pub organization: String,
    pub contact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustodianInfo {
    pub name: String,
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportInfo {
    pub title: String,
    pub date: String,
}

pub fn load_metadata(path: &PathBuf) -> Result<CaseMetadata, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let meta: CaseMetadata = serde_yaml::from_str(&content)?;
    Ok(meta)
}
