pub mod metadata;
pub mod templates;

use crate::models::{Contact, Message, MsgContactTuple};
use crate::processing::unmarshal_msg_contact_tuples_from_json;
use crate::report::metadata::{CaseMetadata, load_metadata};
use crate::report::templates::{
    all_messages_page, chat_thread_page, chats_list_page, contacts_list_page, custodian_page,
    group_chats_page, home_page,
};
use rlibphonenumber::Region;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn is_contact_in_group_chat(
    contact: &crate::models::Contact,
    gc: &crate::processing::GrpChat,
    region: Region,
) -> bool {
    gc.participants.iter().any(|participant| {
        // Normalize the participant identifier (phone or email) using the same rules as contacts.
        let normalized_participant = if participant.contains('@') {
            participant.to_lowercase()
        } else {
            crate::processing::normalize_handle("phone_numbers", participant, region)
        };

        // Check normalized email array
        if let Some(ref emails) = contact.normalized_email_addresses
            && emails.contains(&normalized_participant)
        {
            return true;
        }
        // Check normalized phone array
        if let Some(ref phones) = contact.normalized_phone_numbers
            && phones.contains(&normalized_participant)
        {
            return true;
        }
        false
    })
}

fn copy_attachments(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let src = output_dir.join("evidence/working_copy/Library/Messages/Attachments");
    let dst = output_dir.join("report/attachments");

    if src.exists() {
        if !dst.exists() {
            fs::create_dir_all(&dst)?;
        }
        if let Ok(entries) = fs::read_dir(&src) {
            for entry in entries {
                let entry = entry?;
                let dest_path = dst.join(entry.file_name());
                if entry.file_type()?.is_dir() {
                    // For simplicity in this context, using a recursive copy helper or bash
                    // Since we are in Rust, we'll use a simple recursive function if needed,
                    // but usually, we'd use a crate like `fs_extra`.
                    // To avoid adding dependencies, let's implement a simple recursive copy.
                    copy_dir_all(&entry.path(), &dest_path)?;
                } else {
                    fs::copy(entry.path(), dest_path)?;
                }
            }
        }
    }
    Ok(())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

pub fn generate(
    output_dir: PathBuf,
    meta_path: Option<PathBuf>,
    filtered: bool,
    region: Region,
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
    contact_ids_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_dir = output_dir.join("report");
    fs::create_dir_all(&report_dir)?;

    copy_attachments(&output_dir)?;

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

    // System Metadata
    let generation_time = chrono::Utc::now().to_rfc3339();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let start_date = start.map(|d| d.to_rfc3339());
    let end_date = end.map(|d| d.to_rfc3339());

    let mut filtered_contacts = None;
    if let Some(ids_path) = &contact_ids_file {
        let ids_content = fs::read_to_string(ids_path)?;
        let ids: Vec<String> = ids_content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let json_dir = output_dir.join("json");
        let addresses_json = fs::read_to_string(json_dir.join("addresses.json"))?;
        let addresses: Vec<Contact> = serde_json::from_str(&addresses_json)?;

        let mut names = Vec::new();
        for id in ids {
            if let Some(contact) = addresses
                .iter()
                .find(|c| c.contact_unique_id.as_ref() == Some(&id))
            {
                let first = contact.first_name.as_deref().unwrap_or("");
                let last = contact.last_name.as_deref().unwrap_or("");
                let full_name = format!("{} {}", first, last).trim().to_string();
                if !full_name.is_empty() {
                    names.push(full_name);
                }
            }
        }
        if !names.is_empty() {
            filtered_contacts = Some(names);
        }
    }

    let system_meta = metadata::SystemMetadata {
        generation_time,
        version,
        start_date,
        end_date,
        filtered_contacts,
    };

    // Home page
    let markup = home_page(&meta, &system_meta);
    fs::write(report_dir.join("index.html"), markup.into_string())?;

    // Process contacts and messages
    let json_dir = output_dir.join("json");
    let contacts_json = fs::read_to_string(json_dir.join("addresses.json"))?;
    let contacts: Vec<Contact> = serde_json::from_str(&contacts_json)?;

    let memberships_json = fs::read_to_string(json_dir.join("group_memberships.json"))?;
    let joined_filename = if filtered {
        "filtered.json"
    } else {
        "joined.json"
    };
    let joined_json_str = fs::read_to_string(json_dir.join(joined_filename))?;
    let mut joined_data: Vec<MsgContactTuple> =
        unmarshal_msg_contact_tuples_from_json(&joined_json_str)?;

    let messages: Vec<Message> = joined_data.iter().map(|(m, _)| m.clone()).collect();
    let group_chats =
        crate::processing::process_group_chats_from_json(&memberships_json, &messages, region)?;

    // Sort newest first for global lists
    joined_data.sort_by_key(|a| std::cmp::Reverse(a.0.timestamp));

    let contacts_dir = report_dir.join("contacts");
    fs::create_dir_all(&contacts_dir)?;

    // Contacts listing page
    let list_markup = contacts_list_page(&meta, &contacts);
    fs::write(contacts_dir.join("index.html"), list_markup.into_string())?;

    // Custodian page
    let custodian_messages: Vec<Message> = joined_data
        .iter()
        .filter(|(m, _)| m.is_from_me.unwrap_or(false))
        .map(|(m, _)| m.clone())
        .collect();

    let cust_markup = custodian_page(&meta, &custodian_messages);
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

        let contact_group_chats: Vec<&crate::processing::GrpChat> = group_chats
            .values()
            .filter(|gc| is_contact_in_group_chat(contact, gc, region))
            .collect();

        let detail_markup =
            templates::contact_detail_page(&meta, contact, &contact_messages, &contact_group_chats);
        fs::write(
            contacts_dir.join(contact_filename),
            detail_markup.into_string(),
        )?;
    }

    let chats_dir = report_dir.join("chats");
    fs::create_dir_all(&chats_dir)?;

    let mut chat_groups: HashMap<String, Vec<(Message, Option<Contact>)>> = HashMap::new();
    for (msg, contact) in joined_data.clone() {
        if let Some(chat_id) = msg.chat.clone() {
            let norm_id = crate::processing::normalize_chat_guid(&chat_id);
            chat_groups.entry(norm_id).or_default().push((msg, contact));
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

    // Group Chats summary page
    let group_chats_markup = group_chats_page(&meta, &group_chats, &chat_groups);
    fs::write(
        report_dir.join("group_chats.html"),
        group_chats_markup.into_string(),
    )?;
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
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::models::Contact;
    use crate::processing::GrpChat;
    use serde_json::json;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn generate_uses_filtered_messages_when_filtering_was_applied() {
        let temp_dir = tempdir().expect("create temporary output directory");
        let json_dir = temp_dir.path().join("json");
        fs::create_dir(&json_dir).expect("create JSON directory");
        fs::write(json_dir.join("addresses.json"), "[]").expect("write contacts");
        fs::write(json_dir.join("group_memberships.json"), "[]").expect("write group memberships");
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

        generate(
            temp_dir.path().to_path_buf(),
            None,
            true,
            Region::US,
            None,
            None,
            None,
        )
        .expect("generate filtered report");

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
        fs::write(json_dir.join("group_memberships.json"), "[]").expect("write group memberships");
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

        generate(
            temp_dir.path().to_path_buf(),
            None,
            false,
            Region::US,
            None,
            None,
            None,
        )
        .expect("generate report");

        let report = fs::read_to_string(temp_dir.path().join("report/messages/index.html"))
            .expect("read generated message report");
        assert!(report.contains("unfiltered message"));
    }

    #[test]
    fn test_is_contact_in_group_chat() {
        let contact = Contact {
            contact_id: Some(serde_json::json!(1)),
            contact_unique_id: Some("unique_id".to_string()),
            first_name: Some("John".to_string()),
            middle_name: None,
            last_name: Some("Doe".to_string()),
            nickname: None,
            organization: None,
            job_title: None,
            record_type: None,
            phone_numbers: Some("1234567890 | 9876543210".to_string()),
            email_addresses: Some("john@doe.com".to_string()),
            normalized_email_addresses: None,
            normalized_phone_numbers: Some(vec![
                "+11234567890".to_string(),
                "+19876543210".to_string(),
            ]),
        };

        let mut participants = HashSet::new();
        participants.insert("+11234567890".to_string()); // matches first normalized number
        participants.insert("+19876543210".to_string()); // matches second normalized number

        let gc = GrpChat {
            chat: Some("chat123".to_string()),
            messages: vec![],
            participants,
        };

        assert!(
            is_contact_in_group_chat(&contact, &gc, Region::US),
            "Contact should be matched by normalized phone number"
        );
    }
}
