use crate::models::{Contact, Message};
use crate::report::metadata::CaseMetadata;
use maud::{Markup, html};
use std::collections::HashSet;

pub fn custodian_page(meta: &CaseMetadata, messages: &[Message]) -> Markup {
    let mut emails = HashSet::new();
    let mut phones = HashSet::new();

    for msg in messages {
        if let Some(sender) = &msg.sender {
            if sender.contains('@') {
                emails.insert(sender.clone());
            } else if sender.chars().any(|c| c.is_ascii_digit()) {
                phones.insert(sender.clone());
            }
        }
    }

    let email_list = if emails.is_empty() {
        "N/A".to_string()
    } else {
        emails.into_iter().collect::<Vec<_>>().join(", ")
    };
    let phone_list = if phones.is_empty() {
        "N/A".to_string()
    } else {
        phones.into_iter().collect::<Vec<_>>().join(", ")
    };

    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Custodian - " (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    h1 { "Custodian" }
                    a href="../index.html" { "← Back to Home" }

                    div class="contact-card" {
                        h1 { (meta.custodian.name) }
                        div class="card-grid" {
                            p { "Device: " b { (meta.custodian.device_name) } }
                            p { "Role: " b { "Custodian" } }
                            p { "Phone: " b { (phone_list) } }
                            p { "Email: " b { (email_list) } }
                        }
                    }

                    h2 { "Messages Sent by Custodian" }
                    div class="message-list" {
                        @for msg in messages {
                            div class="message-item" {
                                div class="msg-header" {
                                    span class="timestamp" { (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string())) }
                                    @if let Some(chat_id) = &msg.chat {
                                        span class="chat-link" {
                                            a href=(format!("../chats/chat_{}.html", chat_id)) { "Chat: " (chat_id) }
                                        }
                                    }
                                }
                                div class="text" { (msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }
                            }
                        }
                        @if messages.is_empty() {
                            p { "No messages sent by custodian found." }
                        }
                    }
                }
            }
        }
    }
}

pub fn home_page(meta: &CaseMetadata) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    h1 { (meta.report.title) }

                    div class="metadata-section" {
                        div class="grid" {
                            div class="col" {
                                h2 { "Case Information" }
                                p { "Name: " b { (meta.case.name) } }
                                p { "Number: " b { (meta.case.number) } }
                                p { "Requestor: " b { (meta.case.requestor) } }
                                @if let Some(notes) = &meta.case.notes {
                                    p { "Notes: " i { (notes) } }
                                }
                            }
                            div class="col" {
                                h2 { "Analyst" }
                                p { "Name: " b { (meta.analyst.name) } }
                                p { "Org: " b { (meta.analyst.organization) } }
                                p { "Contact: " b { (meta.analyst.contact) } }
                            }
                            div class="col" {
                                h2 { "Custodian" }
                                p { "Name: " b { (meta.custodian.name) } }
                                p { "Device: " b { (meta.custodian.device_name) } }
                            }
                        }
                    }

                    div class="nav-section" {
                        div class="nav-links" {
                            a href="contacts/index.html" { "View Contacts List" }
                            " | "
                            a href="chats/index.html" { "View Chat Threads" }
                            " | "
                            a href="messages/index.html" { "View All Messages" }
                        }
                    }
                }
            }
        }
    }
}

pub fn contacts_list_page(meta: &CaseMetadata, contacts: &[Contact]) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Contacts - " (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    h1 { "Contacts" }
                    a href="../index.html" { "← Back to Home" }

                    div class="list-container" {
                        ul {
                            li {
                                a href="custodian.html" { "Custodian" }
                            }
                            @for contact in contacts {
                                li {
                                    a href=(format!("contact_{}.html", contact.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                        (contact.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                        " "
                                        (contact.last_name.clone().unwrap_or_default())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn contact_detail_page(meta: &CaseMetadata, contact: &Contact, messages: &[Message]) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (contact.first_name.clone().unwrap_or_else(|| "Contact".to_string())) " - " (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    a href="../index.html" { "← Back to Home" }
                    span { " | " }
                    a href="../contacts/index.html" { "Back to Contacts" }

                    div class="contact-card" {
                        h1 {
                            (contact.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                            " "
                            (contact.last_name.clone().unwrap_or_default())
                        }
                        div class="card-grid" {
                            p { "Phone: " b { (contact.phone_numbers.clone().unwrap_or_else(|| "N/A".to_string())) } }
                            p { "Email: " b { (contact.email_addresses.clone().unwrap_or_else(|| "N/A".to_string())) } }
                            p { "Organization: " b { (contact.organization.clone().unwrap_or_else(|| "N/A".to_string())) } }
                            p { "Job Title: " b { (contact.job_title.clone().unwrap_or_else(|| "N/A".to_string())) } }
                        }
                    }

                    h2 { "Messages" }
                    div class="message-list" {
                        @for msg in messages {
                            div class="message-item" {
                                 div class="msg-header" {
                                     span class="timestamp" { (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string())) }
                                      span class="sender" {
                                          @if msg.is_from_me.unwrap_or(false) {
                                              a href="../contacts/custodian.html" { "Custodian" }
                                          } @else {
                                              (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string()))
                                          }
                                      }

                                     @if let Some(chat_id) = &msg.chat {
                                         span class="chat-link" {
                                             a href=(format!("../chats/chat_{}.html", chat_id)) { "Chat: " (chat_id) }
                                         }
                                     }
                                 }

                                div class="text" { (msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn chats_list_page(meta: &CaseMetadata, chats: &[(String, Message)]) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Chat Threads - " (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    h1 { "Chat Threads" }
                    a href="../index.html" { "← Back to Home" }

                    div class="list-container" {
                        ul {
                            @for (chat_id, last_msg) in chats {
                                li {
                                    div class="chat-summary" {
                                        a href=(format!("chat_{}.html", chat_id)) {
                                            b { (chat_id) }
                                        }
                                        span class="timestamp" { (last_msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string())) }
                                        div class="snippet" { (last_msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn chat_thread_page(
    meta: &CaseMetadata,
    chat_id: &str,
    messages: &[(Message, Option<Contact>)],
) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (chat_id) " - " (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    h1 { "Chat Thread: " (chat_id) }
                    a href="../index.html" { "← Back to Home" }
                    " | "
                    a href="../chats/index.html" { "Back to Chats" }

                    div class="message-list" {
                        @for (msg, contact) in messages {
                            div class="message-item" {
                                 div class="msg-header" {
                                     span class="timestamp" { (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string())) }
                                       span class="sender" {
                                           @if msg.is_from_me.unwrap_or(false) {
                                               a href="../contacts/custodian.html" { "Custodian" }
                                           } @else if let Some(c) = contact {
                                               a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                                   (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                                   " "
                                                   (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                                               }
                                           } @else {
                                               (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string()))
                                           }
                                       }
                                      @if let Some(chat_id) = &msg.chat {
                                          span class="chat-link" {
                                              a href=(format!("../chats/chat_{}.html", chat_id)) { "Chat: " (chat_id) }
                                          }
                                      }
                                 }
                                 div class="text" { (msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }

                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn all_messages_page(
    meta: &CaseMetadata,
    messages: &[(Message, Option<Contact>)],
    current_page: usize,
    total_pages: usize,
) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "All Messages - " (meta.report.title) }
                style { (css()) }
            }
            body {
                div class="container" {
                    h1 { "All Messages" }
                    a href="../index.html" { "← Back to Home" }

                                    div class="message-list" {
                                        @for (msg, contact) in messages {
                                            div class="message-item" {
                                 div class="msg-header" {
                                     span class="timestamp" { (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string())) }
                                     span class="sender" {
                                         @if msg.is_from_me.unwrap_or(false) {
                                             a href="../contacts/custodian.html" { "Custodian" }
                                         } @else if let Some(c) = contact {
                                             a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                                 (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                                 " "
                                                 (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                                             }
                                         } @else {
                                             (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                                     }
                                     @if let Some(chat_id) = &msg.chat {
                                         span class="chat-link" {
                                             a href=(format!("../chats/chat_{}.html", chat_id)) { "Chat: " (chat_id) }
                                         }
                                     }
                                 }
                                 div class="text" { (msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }

                                            }
                                        }
                                    }


                    div class="pagination" {
                        @if current_page > 1 {
                            a href=(format!("page_{}.html", current_page - 1)) { "Previous" }
                        }
                        span { " Page " (current_page) " of " (total_pages) " " }
                        @if current_page < total_pages {
                            a href=(format!("page_{}.html", current_page + 1)) { "Next" }
                        }
                    }
                }
            }
        }
    }
}

fn css() -> String {
    "
    body { font-family: sans-serif; line-height: 1.6; color: #333; max-width: 1000px; margin: 0 auto; padding: 20px; background-color: #f4f4f9; }
    .container { background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
    h1 { color: #2c3e50; text-align: center; border-bottom: 2px solid #eee; padding-bottom: 10px; }
    .metadata-section { margin-bottom: 30px; padding: 20px; background: #f9f9f9; border: 1px solid #ddd; border-radius: 4px; }
    .grid { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px; }
    .col h2 { font-size: 1.2em; color: #555; border-bottom: 1px solid #ccc; }
    .col p { margin: 5px 0; font-size: 0.9em; }
    .nav-section { text-align: center; margin-top: 20px; }
    .nav-links a { font-size: 1.2em; color: #3498db; text-decoration: none; font-weight: bold; }
    .list-container ul { list-style: none; padding: 0; }
    .list-container li { padding: 10px; border-bottom: 1px solid #eee; }
    .list-container a { text-decoration: none; color: #2980b9; font-size: 1.1em; }
    .contact-card { background: #eef2f7; padding: 20px; border-radius: 8px; margin-bottom: 30px; border-left: 5px solid #3498db; }
    .contact-card h1 { text-align: left; border: none; margin-top: 0; }
    .card-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .message-list { display: flex; flex-direction: column; gap: 10px; }
    .message-item { padding: 10px; background: #fff; border: 1px solid #ddd; border-radius: 4px; }
    .msg-header { display: flex; justify-content: space-between; margin-bottom: 5px; font-size: 0.85em; }
    .timestamp { color: #888; }
    .sender a { color: #2980b9; text-decoration: none; font-weight: bold; }
    .chat-link { font-size: 0.8em; color: #7f8c8d; margin-left: 10px; }
    .chat-link a { color: #7f8c8d; text-decoration: none; }
    .chat-link a:hover { text-decoration: underline; }
    .text { white-space: pre-wrap; }
    .pagination { text-align: center; margin-top: 30px; font-weight: bold; }
    .pagination a { color: #3498db; text-decoration: none; padding: 5px 10px; border: 1px solid #3498db; border-radius: 4px; }
    .chat-summary { display: flex; flex-direction: column; }
    .chat-summary b { font-size: 1.1em; color: #2980b9; }
    .chat-summary .timestamp { font-size: 0.8em; margin: 2px 0; }
    .chat-summary .snippet { font-style: italic; color: #666; font-size: 0.9em; }
    ".to_string()
}
