use maud::{html, Markup};
use crate::report::metadata::CaseMetadata;

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
                    
                    div class="content" {
                        p { "Report content will be added here in future versions." }
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
    .content { margin-top: 20px; font-style: italic; color: #666; }
    ".to_string()
}
