pub mod metadata;
pub mod templates;

use crate::models::{Contact, Message, MsgContactTuple};
use crate::processing::unmarshal_msg_contact_tuples_from_json;
use crate::report::metadata::{CaseMetadata, load_metadata};
use crate::report::templates::{
    all_messages_page, chat_thread_page, chats_list_page, contact_detail_page, contacts_list_page,
    custodian_page, home_page,
};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn generate(
    output_dir: PathBuf,
    meta_path: Option<PathBuf>,
    filtered: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let joined_filename = if filtered {
        "filtered.json"
    } else {
        "joined.json"
    };
    let joined_json_str = fs::read_to_string(json_dir.join(joined_filename))?;
    let mut joined_data: Vec<MsgContactTuple> =
        unmarshal_msg_contact_tuples_from_json(&joined_json_str)?;

    // Sort newest first for global lists
    joined_data.sort_by_key(|a| std::cmp::Reverse(a.0.timestamp));

    let contacts_dir = report_dir.join("contacts");
    fs::create_dir_all(&contacts_dir)?;

    // Contacts listing page
    let list_markup = contacts_list_page(&meta, &contacts);
    fs::write(contacts_dir.join("index.html"), list_markup.into_string())?;

    // Custodian page
    let cust_markup = custodian_page(&meta);
    fs::write(
        contacts_dir.join("custodian.html"),
        cust_markup.into_string(),
    )?;

    // Individual contact pages
    for contact in &contacts {
        let contact_id = contact
            .contact_id
            .as_ref()
            .unwrap_or(&serde_json::Value::Null);
        let contact_filename = format!("contact_{}.html", contact_id);

        let contact_messages: Vec<Message> = joined_data
            .iter()
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
        fs::write(
            contacts_dir.join(contact_filename),
            detail_markup.into_string(),
        )?;
    }

    // Chats section
    let chats_dir = report_dir.join("chats");
    fs::create_dir_all(&chats_dir)?;

    let mut chat_groups: HashMap<String, Vec<(Message, Option<Contact>)>> = HashMap::new();
    for (msg, contact) in joined_data.clone() {
        if let Some(chat_id) = msg.chat.clone() {
            chat_groups.entry(chat_id).or_default().push((msg, contact));
        }
    }

    let mut chat_summaries: Vec<(String, Message)> = Vec::new();
    for (chat_id, messages) in &chat_groups {
        if let Some((newest_msg, _)) = messages.iter().max_by_key(|(m, _)| m.timestamp) {
            chat_summaries.push((chat_id.clone(), newest_msg.clone()));
        }
    }
    chat_summaries.sort_by_key(|a| std::cmp::Reverse(a.1.timestamp));

    let chats_list_markup = chats_list_page(&meta, &chat_summaries);
    fs::write(
        chats_dir.join("index.html"),
        chats_list_markup.into_string(),
    )?;

    for (chat_id, messages) in &chat_groups {
        let mut thread_messages = messages.clone();
        // Sort oldest first for the thread
        thread_messages.sort_by_key(|a| a.0.timestamp);

        let thread_markup = chat_thread_page(&meta, chat_id, &thread_messages);
        fs::write(
            chats_dir.join(format!("chat_{}.html", chat_id)),
            thread_markup.into_string(),
        )?;
    }

    // All messages listing with pagination
    let messages_dir = report_dir.join("messages");
    fs::create_dir_all(&messages_dir)?;

    const PAGE_SIZE: usize = 50;
    let total_messages = joined_data.len();
    let total_pages = total_messages.div_ceil(PAGE_SIZE);

    for page in 1..=total_pages {
        let start = (page - 1) * PAGE_SIZE;
        let end = std::cmp::min(start + PAGE_SIZE, total_messages);
        let page_messages = &joined_data[start..end];

        let msg_markup = all_messages_page(&meta, page_messages, page, total_pages);
        let filename = if page == 1 {
            "index.html".to_string()
        } else {
            format!("page_{}.html", page)
        };
        fs::write(messages_dir.join(filename), msg_markup.into_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generate;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn generate_uses_filtered_messages_when_filtering_was_applied() {
        let temp_dir = tempdir().expect("create temporary output directory");
        let json_dir = temp_dir.path().join("json");
        fs::create_dir(&json_dir).expect("create JSON directory");
        fs::write(json_dir.join("addresses.json"), "[]").expect("write contacts");
        fs::write(
            json_dir.join("joined.json"),
            json!([{
                "timestamp": "2024-01-01T00:00:00Z",
                "message": "unfiltered message",
                "contact": null
            }])
            .to_string(),
        )
        .expect("write joined messages");
        fs::write(
            json_dir.join("filtered.json"),
            json!([{
                "timestamp": "2024-01-02T00:00:00Z",
                "message": "filtered message",
                "contact": null
            }])
            .to_string(),
        )
        .expect("write filtered messages");

        generate(temp_dir.path().to_path_buf(), None, true).expect("generate filtered report");

        let report = fs::read_to_string(temp_dir.path().join("report/messages/index.html"))
            .expect("read generated message report");
        assert!(report.contains("filtered message"));
        assert!(!report.contains("unfiltered message"));
    }

    #[test]
    fn generate_uses_joined_messages_when_filtering_was_not_applied() {
        let temp_dir = tempdir().expect("create temporary output directory");
        let json_dir = temp_dir.path().join("json");
        fs::create_dir(&json_dir).expect("create JSON directory");
        fs::write(json_dir.join("addresses.json"), "[]").expect("write contacts");
        fs::write(
            json_dir.join("joined.json"),
            json!([{
                "timestamp": "2024-01-01T00:00:00Z",
                "message": "unfiltered message",
                "contact": null
            }])
            .to_string(),
        )
        .expect("write joined messages");

        generate(temp_dir.path().to_path_buf(), None, false).expect("generate report");

        let report = fs::read_to_string(temp_dir.path().join("report/messages/index.html"))
            .expect("read generated message report");
        assert!(report.contains("unfiltered message"));
    }
}
