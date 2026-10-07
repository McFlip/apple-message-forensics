pub mod metadata;
pub mod templates;

use std::fs;
use std::path::PathBuf;
use crate::report::metadata::{load_metadata, CaseMetadata};
use crate::report::templates::home_page;

pub fn generate(output_dir: PathBuf, meta_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let report_dir = output_dir.join("report");
    fs::create_dir_all(&report_dir)?;

    let meta = if let Some(path) = meta_path {
        load_metadata(&path)?
    } else {
        // Fallback empty metadata if none provided
        CaseMetadata {
            case: metadata::CaseInfo {
                name: "Unknown".to_string(),
                number: "Unknown".to_string(),
                requestor: "Unknown".to_string(),
                notes: None,
            },
            analyst: metadata::AnalystInfo {
                name: "Unknown".to_string(),
                organization: "Unknown".to_string(),
                contact: "Unknown".to_string(),
            },
            custodian: metadata::CustodianInfo {
                name: "Unknown".to_string(),
                device_name: "Unknown".to_string(),
            },
            report: metadata::ReportInfo {
                title: "iMessage Forensic Report".to_string(),
                date: "Unknown".to_string(),
            },
        }
    };

    let markup = home_page(&meta);
    fs::write(report_dir.join("index.html"), markup.into_string())?;

    Ok(())
}
