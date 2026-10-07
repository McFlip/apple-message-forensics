use maud::{html, Markup};
use crate::report::metadata::CaseMetadata;
use crate::models::{Contact, Message};

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
                        a href="contacts/index.html" { "View Contacts List" }
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
                            @for contact in contacts {
                                li {
                                    a href=(format!("contact_{}.html", contact.contact_id.as_ref().unwrap_or(&serde_json::Value::Null))) {
                                        (contact.first_name.clone().unwrap_or_else(|| "Unknown".to_string()))
                                        " "
                                        (contact.last_name.clone().unwrap_or_else(|| "".to_string()))
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
                            (contact.last_name.clone().unwrap_or_else(|| "".to_string())) 
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
                                span class="timestamp" { (msg.timestamp.clone().map(|t| t.to_rfc3339()).unwrap_or_else(|| "Unknown".to_string())) }
                                div class="text" { (msg.message.clone().unwrap_or_else(|| "[No content]".to_string())) }
                            }
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
    .nav-section a { font-size: 1.2em; color: #3498db; text-decoration: none; font-weight: bold; }
    .list-container ul { list-style: none; padding: 0; }
    .list-container li { padding: 10px; border-bottom: 1px solid #eee; }
    .list-container a { text-decoration: none; color: #2980b9; font-size: 1.1em; }
    .contact-card { background: #eef2f7; padding: 20px; border-radius: 8px; margin-bottom: 30px; border-left: 5px solid #3498db; }
    .contact-card h1 { text-align: left; border: none; margin-top: 0; }
    .card-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .message-list { display: flex; flex-direction: column; gap: 10px; }
    .message-item { padding: 10px; background: #fff; border: 1px solid #ddd; border-radius: 4px; }
    .timestamp { font-size: 0.8em; color: #888; display: block; margin-bottom: 5px; }
    .text { white-space: pre-wrap; }
    ".to_string()
}
