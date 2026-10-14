use crate::models::{Contact, ContactLookup, Message, MsgContactTuple};
use chrono::{DateTime, Utc};
use rlibphonenumber::{PhoneNumber, Region};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Statistics {
    pub total_messages: usize,
    pub total_attachments: usize,
    pub top_talkers: Vec<TopTalker>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TopTalker {
    pub name: String,
    pub message_count: usize,
    pub contact_id: Option<serde_json::Value>,
}

pub fn statistics_from_json(json_str: &str) -> Result<String, serde_json::Error> {
    let messages = unmarshal_msg_contact_tuples_from_json(json_str)?;
    let mut talkers: HashMap<String, (String, usize, Option<serde_json::Value>)> = HashMap::new();

    for (message, contact) in &messages {
        let Some(contact) = contact else {
            continue;
        };

        let contact_name = [
            contact.first_name.as_deref().unwrap_or("").trim(),
            contact.last_name.as_deref().unwrap_or("").trim(),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
        let name = if contact_name.is_empty() {
            message
                .sender
                .as_deref()
                .filter(|sender| !sender.is_empty())
                .unwrap_or("Unknown")
                .to_owned()
        } else {
            contact_name.clone()
        };
        let identity = contact
            .contact_unique_id
            .clone()
            .or_else(|| (!contact_name.is_empty()).then_some(contact_name))
            .or_else(|| message.sender.clone())
            .unwrap_or_else(|| name.clone());
        let entry = talkers
            .entry(identity)
            .or_insert((name, 0, contact.contact_id.clone()));
        entry.1 += 1;
    }

    let mut top_talkers: Vec<TopTalker> = talkers
        .into_values()
        .map(|(name, message_count, contact_id)| TopTalker {
            name,
            message_count,
            contact_id,
        })
        .collect();
    top_talkers.sort_by(|a, b| {
        b.message_count
            .cmp(&a.message_count)
            .then_with(|| a.name.cmp(&b.name))
    });
    top_talkers.truncate(10);

    serde_json::to_string(&Statistics {
        total_messages: messages.len(),
        total_attachments: messages
            .iter()
            .filter(|(message, _)| {
                message
                    .attachment
                    .as_deref()
                    .is_some_and(|attachment| !attachment.trim().is_empty())
            })
            .count(),
        top_talkers,
    })
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GrpChat {
    pub chat: Option<String>,
    pub messages: Vec<Message>,
    pub participants: HashSet<String>,
}

pub fn is_group_chat(chat_handle: &str) -> bool {
    // Chat GUIDs can contain semicolon‑separated components (e.g. "any;+;chat157232977552461316").
    // The actual chat identifier is the part after the last ';' that starts with "chat" followed by digits.
    let parts: Vec<&str> = chat_handle.split(';').collect();
    if let Some(last) = parts.last()
        && let Some(digits) = last.strip_prefix("chat")
    {
        return !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit());
    }
    false
}

pub fn normalize_handle(field: &str, value: &str, region: Region) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    if field == "email_addresses" {
        value.to_lowercase()
    } else {
        // Use rlibphonenumber to parse and format to E.164.
        // First try parsing without an explicit region (handles numbers with a leading '+').
        // If that fails, fall back to assuming US numbers.
        let parsed =
            PhoneNumber::parse(value, None).or_else(|_| PhoneNumber::parse(value, Some(region)));
        match parsed {
            Ok(num) => format!("{}", num),
            Err(_) => String::new(),
        }
    }
}

// Extract the canonical chat identifier (last segment) from a GUID like "any;+;chat123".
pub fn normalize_chat_guid(guid: &str) -> String {
    guid.split(';').next_back().unwrap_or(guid).to_string()
}

pub fn process_group_chats_from_json(
    memberships_json: &str,
    messages: &[Message],
    region: Region,
) -> Result<HashMap<String, GrpChat>, String> {
    #[derive(serde::Deserialize)]
    struct GroupMembership {
        chat_guid: String,
        participants: Vec<String>,
    }

    let memberships: Vec<GroupMembership> = serde_json::from_str(memberships_json)
        .map_err(|err| format!("cannot parse group memberships JSON: {}", err))?;

    let mut group_chats = HashMap::new();

    for membership in memberships {
        if is_group_chat(&membership.chat_guid) {
            let normalized_guid = normalize_chat_guid(&membership.chat_guid);
            let chat_messages: Vec<Message> = messages
                .iter()
                .filter(|m| {
                    if let Some(chat) = &m.chat {
                        normalize_chat_guid(chat) == normalized_guid
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            group_chats.insert(
                normalized_guid.clone(),
                GrpChat {
                    chat: Some(normalized_guid),
                    messages: chat_messages,
                    participants: {
                        // Normalize each participant (phone numbers or email addresses) using the existing helper.
                        // Email addresses are lower‑cased; phone numbers are normalized via normalize_handle.
                        let mut set = std::collections::HashSet::new();
                        for p in membership.participants {
                            let normalized = if p.contains('@') {
                                p.to_lowercase()
                            } else {
                                normalize_handle("phone_numbers", &p, region)
                            };
                            if !normalized.is_empty() {
                                set.insert(normalized);
                            }
                        }
                        set
                    },
                },
            );
        }
    }

    Ok(group_chats)
}

pub fn unmarshal_addresses_from_json(
    json_str: &str,
    region: Region,
) -> Result<ContactLookup, String> {
    let contacts: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|err| format!("cannot parse addresses JSON: {}", err))?;
    let mut addresses = HashMap::new();

    for (contact_index, contact) in contacts.iter().enumerate() {
        let contact_object = contact
            .as_object()
            .ok_or_else(|| format!("address at index {} is not a JSON object", contact_index))?;

        let mut contact_struct: Contact =
            serde_json::from_value(contact.clone()).map_err(|err| {
                format!(
                    "cannot deserialize address at index {}: {}",
                    contact_index, err
                )
            })?;
        // Collect normalized handles for this contact
        let mut norm_emails: Vec<String> = Vec::new();
        let mut norm_phones: Vec<String> = Vec::new();
        let mut all_keys: Vec<String> = Vec::new();

        for field in ["email_addresses", "phone_numbers"] {
            let Some(value) = contact_object.get(field) else {
                continue;
            };
            let Some(values) = value.as_str() else {
                if value.is_null() {
                    continue;
                }
                return Err(format!(
                    "address at index {} has a non-string {} field",
                    contact_index, field
                ));
            };

            for raw in values.split('|').map(str::trim).filter(|v| !v.is_empty()) {
                let normalized = normalize_handle(field, raw, region);
                if !normalized.is_empty() {
                    all_keys.push(normalized.clone());
                }
                // Populate per‑contact normalized vectors
                if field == "email_addresses" {
                    if !normalized.is_empty() {
                        norm_emails.push(normalized);
                    }
                } else {
                    if !normalized.is_empty() {
                        norm_phones.push(normalized);
                    }
                }
            }
        }
        // Assign vectors to the contact struct (None when empty)
        if !norm_emails.is_empty() {
            contact_struct.normalized_email_addresses = Some(norm_emails);
        }
        if !norm_phones.is_empty() {
            contact_struct.normalized_phone_numbers = Some(norm_phones);
        }
        // Insert the contact into the lookup map for each normalized key
        for key in all_keys {
            addresses.insert(key, contact_struct.clone());
        }
    }

    Ok(addresses)
}

pub fn unmarshal_messages_from_json(json_str: &str) -> Result<Vec<Message>, String> {
    #[derive(serde::Deserialize)]
    struct SerializedMessage {
        timestamp: Option<String>,
        date_edited: Option<String>,
        chat: Option<String>,
        sender: Option<String>,
        is_from_me: Option<bool>,
        message: Option<String>,
        attachment: Option<String>,
        is_delivered: Option<bool>,
        is_sent: Option<bool>,
        is_read: Option<bool>,
        is_forward: Option<bool>,
    }

    let serialized_messages: Vec<SerializedMessage> = serde_json::from_str(json_str)
        .map_err(|err| format!("cannot parse messages JSON: {}", err))?;

    serialized_messages
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            let timestamp = message
                .timestamp
                .as_deref()
                .map(|timestamp| {
                    DateTime::parse_from_rfc3339(timestamp)
                        .map(|timestamp| timestamp.with_timezone(&Utc))
                        .map_err(|err| {
                            format!(
                                "cannot parse timestamp for message at index {}: {}",
                                index, err
                            )
                        })
                })
                .transpose()?;

            let date_edited = message
                .date_edited
                .as_deref()
                .map(|timestamp| {
                    DateTime::parse_from_rfc3339(timestamp)
                        .map(|timestamp| timestamp.with_timezone(&Utc))
                        .map_err(|err| {
                            format!(
                                "cannot parse date_edited for message at index {}: {}",
                                index, err
                            )
                        })
                })
                .transpose()?;

            Ok(Message {
                timestamp,
                date_edited,
                chat: message.chat,
                sender: message.sender,
                is_from_me: message.is_from_me,
                message: message.message,
                attachment: message.attachment,
                is_delivered: message.is_delivered,
                is_sent: message.is_sent,
                is_read: message.is_read,
                is_forward: message.is_forward,
            })
        })
        .collect()
}

pub fn unmarshal_msg_contact_tuples_from_json(
    json_str: &str,
) -> Result<Vec<MsgContactTuple>, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct SerializedJoinedMessage {
        timestamp: Option<DateTime<Utc>>,
        date_edited: Option<DateTime<Utc>>,
        chat: Option<String>,
        sender: Option<String>,
        is_from_me: Option<bool>,
        message: Option<String>,
        attachment: Option<String>,
        is_delivered: Option<bool>,
        is_sent: Option<bool>,
        is_read: Option<bool>,
        is_forward: Option<bool>,
        contact: Option<Contact>,
    }

    let joined_messages: Vec<SerializedJoinedMessage> = serde_json::from_str(json_str)?;

    Ok(joined_messages
        .into_iter()
        .map(|joined| {
            (
                Message {
                    timestamp: joined.timestamp,
                    date_edited: joined.date_edited,
                    chat: joined.chat,
                    sender: joined.sender,
                    is_from_me: joined.is_from_me,
                    message: joined.message,
                    attachment: joined.attachment,
                    is_delivered: joined.is_delivered,
                    is_sent: joined.is_sent,
                    is_read: joined.is_read,
                    is_forward: joined.is_forward,
                },
                joined.contact,
            )
        })
        .collect())
}

pub fn join_messages_to_contacts(
    messages: &[Message],
    addresses: &ContactLookup,
) -> Result<Vec<MsgContactTuple>, String> {
    messages
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, message)| {
            let sender_key = match message.sender.as_deref() {
                None => None,
                Some(sender) => {
                    if sender.contains('@') {
                        Some(sender.to_lowercase())
                    } else {
                        let is_from_me = message.is_from_me.ok_or_else(|| {
                            format!("message at index {} is missing is_from_me", index)
                        })?;
                        let unclean_digits = if is_from_me {
                            message.chat.clone().ok_or_else(|| {
                                format!(
                                    "message at index {} from custodian is missing chat handle",
                                    index
                                )
                            })?
                        } else {
                            sender.to_string()
                        };
                        let digits = unclean_digits
                            .chars()
                            .filter(char::is_ascii_digit)
                            .collect::<String>();
                        if digits.is_empty() {
                            None
                        } else if digits.starts_with('1') {
                            Some(format!("+{}", digits))
                        } else {
                            Some(format!("+1{}", digits))
                        }
                    }
                }
            };
            let contact = sender_key.and_then(|key| addresses.get(&key).cloned());
            Ok((message, contact))
        })
        .collect()
}

pub fn marshal_msg_contact_tuple_to_json(joined: &[MsgContactTuple]) -> serde_json::Value {
    serde_json::Value::Array(
        joined
            .iter()
            .map(|(message, contact)| {
                serde_json::json!({
                    "timestamp": message.timestamp.as_ref().map(|timestamp| {
                        timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    }),
                    "date_edited": message.date_edited.as_ref().map(|timestamp| {
                        timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    }),
                    "chat": message.chat,
                    "sender": message.sender,
                    "is_from_me": message.is_from_me,
                    "message": message.message,
                    "attachment": message.attachment,
                    "is_delivered": message.is_delivered,
                    "is_sent": message.is_sent,
                    "is_read": message.is_read,
                    "is_forward": message.is_forward,
                    "contact": contact,
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_handle_phone() {
        assert_eq!(
            normalize_handle("phone_numbers", "1234567890", Region::US),
            "+11234567890"
        );
        assert_eq!(
            normalize_handle("phone_numbers", "+11234567890", Region::US),
            "+11234567890"
        );
        assert_eq!(
            normalize_handle("phone_numbers", "234567890", Region::US),
            "+1234567890"
        );
    }

    #[test]
    fn test_normalize_handle_email() {
        assert_eq!(
            normalize_handle("email_addresses", "JOHN@DOE.COM", Region::US),
            "john@doe.com"
        );
    }

    #[test]
    fn statistics_count_messages_attachments_and_top_talkers() {
        let joined = json!([
            {
                "message": "hello",
                "attachment": "photo.jpg",
                "sender": "+15550000001",
                "contact": {
                    "contact_unique_id": "contact-1",
                    "contact_id": 42,
                    "first_name": "Alex",
                    "last_name": "Doe"
                }
            },
            {
                "message": "reply",
                "sender": "+15550000001",
                "contact": {
                    "contact_unique_id": "contact-1",
                    "contact_id": 42,
                    "first_name": "Alex",
                    "last_name": "Doe"
                }
            },
            {
                "message": "unknown",
                "contact": null
            }
        ])
        .to_string();

        let statistics: Statistics =
            serde_json::from_str(&statistics_from_json(&joined).expect("calculate statistics"))
                .expect("parse statistics JSON");
        assert_eq!(statistics.total_messages, 3);
        assert_eq!(statistics.total_attachments, 1);
        assert_eq!(
            statistics.top_talkers,
            vec![TopTalker {
                name: "Alex Doe".to_string(),
                message_count: 2,
                contact_id: Some(json!(42)),
            }]
        );
    }

    #[test]
    fn statistics_limit_top_talkers_to_ten_contacts() {
        let joined: Vec<serde_json::Value> = (0..11)
            .map(|index| {
                json!({
                    "sender": format!("contact-{index}"),
                    "contact": {
                        "contact_unique_id": format!("contact-{index}"),
                        "first_name": format!("Contact {index}")
                    }
                })
            })
            .collect();

        let statistics: Statistics = serde_json::from_str(
            &statistics_from_json(&serde_json::to_string(&joined).expect("serialize messages"))
                .expect("calculate statistics"),
        )
        .expect("parse statistics JSON");
        assert_eq!(statistics.top_talkers.len(), 10);
    }
}
