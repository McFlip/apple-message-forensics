pub mod metadata;
pub mod templates;

use std::fs;
use std::path::PathBuf;
use crate::report::metadata::{load_metadata, CaseMetadata};
use crate::report::templates::{home_page, contacts_list_page, contact_detail_page};
use crate::models::{Contact, Message};
use serde_json;
use chrono::{DateTime, Utc};

#[derive(serde::Deserialize)]
struct JoinedRecord {
    timestamp: Option<String>,
    chat: Option<String>,
    sender: Option<String>,
    message: Option<String>,
    attachment: Option<String>,
    contact: Option<Contact>,
}

pub fn generate(output_dir: PathBuf, meta_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let report_dir = output_dir.join("report");
    fs::create_dir_all(&report_dir)?;

    let meta = if let Some(path) = meta_path {
        load_metadata(&path)?
    } else {
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

    // Home page
    let markup = home_page(&meta);
    fs::write(report_dir.join("index.html"), markup.into_string())?;

    // Process contacts and messages
    let json_dir = output_dir.join("json");
    let contacts_json = fs::read_to_string(json_dir.join("addresses.json"))?;
    let contacts: Vec<Contact> = serde_json::from_str(&contacts_json)?;

    let joined_json_str = fs::read_to_string(json_dir.join("joined.json"))?;
    let joined_records: Vec<JoinedRecord> = serde_json::from_str(&joined_json_str)?;
    
    let joined_data: Vec<(Message, Option<Contact>)> = joined_records.into_iter().map(|r| {
        let timestamp = r.timestamp.as_deref().and_then(|t| {
            DateTime::parse_from_rfc3339(t).ok().map(|dt| dt.with_timezone(&Utc))
        });
        
        (
            Message {
                timestamp,
                chat: r.chat,
                sender: r.sender,
                message: r.message,
                attachment: r.attachment,
            },
            r.contact,
        )
    }).collect();

    let contacts_dir = report_dir.join("contacts");
    fs::create_dir_all(&contacts_dir)?;

    // Contacts listing page
    let list_markup = contacts_list_page(&meta, &contacts);
    fs::write(contacts_dir.join("index.html"), list_markup.into_string())?;

    // Individual contact pages
    for contact in &contacts {
        let contact_id = contact.contact_id.as_ref().unwrap_or(&serde_json::Value::Null);
        let contact_filename = format!("contact_{}.html", contact_id);
        
        let contact_messages: Vec<Message> = joined_data.iter()
            .filter(|(_, c)| {
                if let Some(c) = c {
                    c.contact_id == contact.contact_id
                } else {
                    false
                }
            })
            .map(|(m, _)| m.clone())
            .collect();

        let detail_markup = contact_detail_page(&meta, contact, &contact_messages);
        fs::write(contacts_dir.join(contact_filename), detail_markup.into_string())?;
    }

    Ok(())
}
