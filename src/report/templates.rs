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

    page_layout(
        &format!("Custodian - {}", meta.report.title),
        html! {
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
                    (render_message_item(msg, None))
                }
                @if messages.is_empty() {
                    p { "No messages sent by custodian found." }
                }
            }
        },
    )
}

pub fn home_page(meta: &CaseMetadata) -> Markup {
    page_layout(
        &meta.report.title,
        html! {
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
        },
    )
}

pub fn contacts_list_page(meta: &CaseMetadata, contacts: &[Contact]) -> Markup {
    page_layout(
        &format!("Contacts - {}", meta.report.title),
        html! {
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
        },
    )
}

pub fn contact_detail_page(meta: &CaseMetadata, contact: &Contact, messages: &[Message]) -> Markup {
    page_layout(
        &format!(
            "{} - {}",
            contact
                .first_name
                .clone()
                .unwrap_or_else(|| "Contact".to_string()),
            meta.report.title
        ),
        html! {
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
                    (render_message_item(msg, Some(contact)))
                }
            }
        },
    )
}

pub fn chats_list_page(meta: &CaseMetadata, chats: &[(String, Message)]) -> Markup {
    page_layout(
        &format!("Chat Threads - {}", meta.report.title),
        html! {
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
        },
    )
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
                                    link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css";
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
                                     span class="timestamp" {
                                         (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string()))
                                         @if let Some(edited) = &msg.date_edited {
                                             span class="edited-info" {
                                                 i class="fa-solid fa-pen" { }
                                                 " Edited: " (edited.to_rfc3339())
                                             }
                                         }
                                     }
                                      div class="msg-status" {
                                          @if msg.is_from_me.unwrap_or(false) && !msg.is_sent.unwrap_or(true) {
                                              i class="fa-solid fa-pen-to-square" title="Draft" { }
                                          }
                                          @if msg.is_forward.unwrap_or(false) {
                                              i class="fa-solid fa-share-from-square" title="Forwarded" { }
                                          }
                                          @if msg.is_delivered.unwrap_or(false) {
                                              i class="fa-solid fa-check-double" title="Delivered" { }
                                          }
                                          @if msg.is_read.unwrap_or(false) {
                                              i class="fa-solid fa-eye" title="Read" { }
                                          }
                                      }
                                      div class="sender-flow" {
                                          @if msg.is_from_me.unwrap_or(false) {
                                              a href="../contacts/custodian.html" { "Custodian" }
                                              i class="fa-solid fa-arrow-right" { }
                                              @if let Some(c) = contact {
                                                  a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                                      (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                                      " "
                                                      (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                                                  }
                                              } @else {
                                                  span { (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                                              }
                                          } @else {
                                              @if let Some(c) = contact {
                                                  a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                                      (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                                      " "
                                                      (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                                                  }
                                              } @else {
                                                  span { (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                                              }
                                              i class="fa-solid fa-arrow-right" { }
                                              a href="../contacts/custodian.html" { "Custodian" }
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
                link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css";
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
                                      span class="timestamp" {
                                          (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string()))
                                          @if let Some(edited) = &msg.date_edited {
                                              span class="edited-info" {
                                                  i class="fa-solid fa-pen" { }
                                                  " Edited: " (edited.to_rfc3339())
                                              }
                                          }
                                      }
                                       div class="msg-status" {
                                           @if msg.is_from_me.unwrap_or(false) && !msg.is_sent.unwrap_or(true) {
                                               i class="fa-solid fa-pen-to-square" title="Draft" { }
                                           }
                                           @if msg.is_forward.unwrap_or(false) {
                                               i class="fa-solid fa-share-from-square" title="Forwarded" { }
                                           }
                                           @if msg.is_delivered.unwrap_or(false) {
                                               i class="fa-solid fa-check-double" title="Delivered" { }
                                           }
                                           @if msg.is_read.unwrap_or(false) {
                                               i class="fa-solid fa-eye" title="Read" { }
                                           }
                                       }
                                       div class="sender-flow" {
                                           @if msg.is_from_me.unwrap_or(false) {
                                               a href="../contacts/custodian.html" { "Custodian" }
                                               i class="fa-solid fa-arrow-right" { }
                                               @if let Some(c) = contact {
                                                   a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                                       (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                                       " "
                                                       (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                                                   }
                                               } @else {
                                                   span { (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                                               }
                                           } @else {
                                               @if let Some(c) = contact {
                                                   a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                                       (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                                       " "
                                                       (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                                                   }
                                               } @else {
                                                   span { (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                                               }
                                               i class="fa-solid fa-arrow-right" { }
                                               a href="../contacts/custodian.html" { "Custodian" }
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
    r#"
    body { font-family: sans-serif; line-height: 1.6; color: #333; background-color: #f4f4f9; margin: 0; padding: 20px; }
    .container { max-width: 1000px; margin: auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
    h1, h2 { color: #2c3e50; }
    .metadata-section { display: flex; gap: 20px; margin-bottom: 30px; padding: 20px; background: #eef2f7; border-radius: 8px; }
    .col { flex: 1; }
    .nav-section { text-align: center; margin-bottom: 30px; }
    .nav-links a { margin: 0 10px; text-decoration: none; color: #3498db; font-weight: bold; }
    .contact-card { background: #f9f9f9; padding: 20px; border-left: 5px solid #3498db; margin-bottom: 20px; }
    .card-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .list-container ul { list-style: none; padding: 0; }
    .list-container li { margin: 10px 0; }
    .list-container a { text-decoration: none; color: #3498db; }
    .message-list { margin-top: 20px; }
    .message-item { border-bottom: 1px solid #eee; padding: 15px 0; }
    .msg-header { display: flex; gap: 15px; align-items: center; font-size: 0.9em; color: #666; margin-bottom: 8px; }
    .timestamp { font-family: monospace; }
    .edited-info { color: #999; font-style: italic; margin-left: 5px; }
    .msg-status { display: flex; gap: 8px; }
    .msg-status i { cursor: help; }
    .sender-flow { font-weight: bold; color: #444; display: flex; align-items: center; gap: 5px; }
    .sender-flow a { text-decoration: none; color: #2c3e50; }
    .sender-flow i { font-size: 0.8em; color: #999; }
    .chat-link { font-size: 0.8em; }
    .chat-link a { text-decoration: none; color: #3498db; }
    .text { margin-left: 10px; white-space: pre-wrap; }
    .chat-summary { display: flex; align-items: center; gap: 15px; padding: 10px; border-bottom: 1px solid #eee; }
    .chat-summary b { min-width: 150px; }
    .chat-summary .timestamp { color: #999; font-size: 0.8em; }
    .chat-summary .snippet { flex: 1; color: #666; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .pagination { margin-top: 20px; text-align: center; }
    .pagination a { margin: 0 10px; text-decoration: none; color: #3498db; }
    "#.to_string()
}

fn page_layout(title: &str, content: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (title) }
                link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css";
                style { (css()) }
            }
            body {
                div class="container" {
                    (content)
                }
            }
        }
    }
}

fn render_message_header(msg: &Message, contact: Option<&Contact>) -> Markup {
    html! {
        div class="msg-header" {
            span class="timestamp" {
                (msg.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string()))
                @if let Some(edited) = &msg.date_edited {
                    span class="edited-info" {
                        i class="fa-solid fa-pen" { }
                        " Edited: " (edited.to_rfc3339())
                    }
                }
            }
            div class="msg-status" {
                @if msg.is_from_me.unwrap_or(false) && !msg.is_sent.unwrap_or(true) {
                    i class="fa-solid fa-pen-to-square" title="Draft" { }
                }
                @if msg.is_forward.unwrap_or(false) {
                    i class="fa-solid fa-share-from-square" title="Forwarded" { }
                }
                @if msg.is_delivered.unwrap_or(false) {
                    i class="fa-solid fa-check-double" title="Delivered" { }
                }
                @if msg.is_read.unwrap_or(false) {
                    i class="fa-solid fa-eye" title="Read" { }
                }
            }
            div class="sender-flow" {
                @if msg.is_from_me.unwrap_or(false) {
                    a href="../contacts/custodian.html" { "Custodian" }
                    i class="fa-solid fa-arrow-right" { }
                    @if let Some(c) = contact {
                        a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                            (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                            " "
                            (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                        }
                    } @else {
                        span { (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                    }
                } @else {
                    @if let Some(c) = contact {
                        a href=(format!("../contacts/contact_{}.html", c.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                            (c.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                            " "
                            (c.last_name.clone().unwrap_or_else(|| "".to_string()))
                        }
                    } @else {
                        span { (msg.sender.clone().unwrap_or_else(|| "Unknown".to_string())) }
                    }
                    i class="fa-solid fa-arrow-right" { }
                    a href="../contacts/custodian.html" { "Custodian" }
                }
            }
            @if let Some(chat_id) = &msg.chat {
                span class="chat-link" {
                    a href=(format!("../chats/chat_{}.html", chat_id)) { "Chat: " (chat_id) }
                }
            }
        }
    }
}

fn render_message_item(msg: &Message, contact: Option<&Contact>) -> Markup {
    html! {
        div class="message-item" {
            (render_message_header(msg, contact))
            div class="text" { (msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }
        }
    }
}
