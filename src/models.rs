use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    pub contact_id: Option<serde_json::Value>,
    pub contact_unique_id: Option<String>,
    pub email_addresses: Option<String>,
    pub first_name: Option<String>,
    pub job_title: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub nickname: Option<String>,
    pub organization: Option<String>,
    pub phone_numbers: Option<String>,
    pub record_type: Option<serde_json::Value>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub timestamp: Option<DateTime<Utc>>,
    pub date_edited: Option<DateTime<Utc>>,
    pub chat: Option<String>,
    pub sender: Option<String>,
    pub is_from_me: Option<bool>,
    pub message: Option<String>,
    pub attachment: Option<String>,
    pub is_delivered: Option<bool>,
    pub is_sent: Option<bool>,
    pub is_read: Option<bool>,
    pub is_forward: Option<bool>,
}

pub type EmailOrPhone = String;
pub type ContactLookup = HashMap<EmailOrPhone, Contact>;
pub type MsgContactTuple = (Message, Option<Contact>);
