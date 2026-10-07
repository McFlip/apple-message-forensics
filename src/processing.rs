use crate::models::{Contact, ContactLookup, Message, MsgContactTuple};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub fn unmarshal_addresses_from_json(json_str: &str) -> Result<ContactLookup, String> {
    let contacts: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|err| format!("cannot parse addresses JSON: {}", err))?;
    let mut addresses = HashMap::new();

    for (contact_index, contact) in contacts.iter().enumerate() {
        let contact_object = contact
            .as_object()
            .ok_or_else(|| format!("address at index {} is not a JSON object", contact_index))?;

        let contact_struct: Contact = serde_json::from_value(contact.clone()).map_err(|err| {
            format!(
                "cannot deserialize address at index {}: {}",
                contact_index, err
            )
        })?;

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

            for value in values
                .split('|')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let key = if field == "email_addresses" {
                    value.to_lowercase()
                } else {
                    let digits = value
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>();
                    match digits.chars().next() {
                        Some('1') => format!("+{}", digits),
                        Some('+') => digits,
                        Some(_) => format!("+1{}", digits),
                        None => String::new(),
                    }
                };
                if !key.is_empty() {
                    addresses.insert(key, contact_struct.clone());
                }
            }
        }
    }

    Ok(addresses)
}

pub fn unmarshal_messages_from_json(json_str: &str) -> Result<Vec<Message>, String> {
    #[derive(serde::Deserialize)]
    struct SerializedMessage {
        timestamp: Option<String>,
        chat: Option<String>,
        sender: Option<String>,
        message: Option<String>,
        attachment: Option<String>,
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

            Ok(Message {
                timestamp,
                chat: message.chat,
                sender: message.sender,
                message: message.message,
                attachment: message.attachment,
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
        chat: Option<String>,
        sender: Option<String>,
        message: Option<String>,
        attachment: Option<String>,
        contact: Option<Contact>,
    }

    let joined_messages: Vec<SerializedJoinedMessage> = serde_json::from_str(json_str)?;

    Ok(joined_messages
        .into_iter()
        .map(|joined| {
            (
                Message {
                    timestamp: joined.timestamp,
                    chat: joined.chat,
                    sender: joined.sender,
                    message: joined.message,
                    attachment: joined.attachment,
                },
                joined.contact,
            )
        })
        .collect())
}

pub fn join_messages_to_contacts(
    messages: &[Message],
    addresses: &ContactLookup,
) -> Vec<MsgContactTuple> {
    messages
        .iter()
        .cloned()
        .map(|message| {
            let sender_key = message.sender.as_deref().and_then(|sender| {
                if sender.contains('@') {
                    Some(sender.to_lowercase())
                } else {
                    let digits = sender
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
            });
            let contact = sender_key.and_then(|key| addresses.get(&key).cloned());
            (message, contact)
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
                    "chat": message.chat,
                    "sender": message.sender,
                    "message": message.message,
                    "attachment": message.attachment,
                    "contact": contact,
                })
            })
            .collect(),
    )
}
