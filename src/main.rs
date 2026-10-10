use apple_message_forensics::check_source;
use apple_message_forensics::logger;
use apple_message_forensics::setup_output_dir;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the output directory
    #[arg(short, long, default_value = "output", global = true)]
    output: PathBuf,

    /// Path to the case metadata YAML file
    #[arg(short, long, global = true)]
    meta: Option<PathBuf>,

    /// Include messages on or after this RFC 3339 date and time
    #[arg(long, value_name = "DATE", global = true)]
    start: Option<DateTime<Utc>>,

    /// Include messages on or before this RFC 3339 date and time
    #[arg(long, value_name = "DATE", global = true)]
    end: Option<DateTime<Utc>>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Only generate a report from existing collection
    Report,
    /// Collect everything and generate report (default behavior)
    Collect,
}

fn default_home_dir() -> PathBuf {
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home),
        None => {
            logger::error("HOME environment variable is not set");
            process::exit(1);
        }
    }
}

fn chat_db_path_from_src_dir(src_dir: &Path) -> PathBuf {
    src_dir.join("chat.db")
}

fn filter_joined_messages(
    json_dir: &Path,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Result<bool, Box<dyn Error>> {
    if start.is_none() && end.is_none() {
        return Ok(false);
    }

    let joined_json = std::fs::read_to_string(json_dir.join("joined.json"))?;
    let joined = apple_message_forensics::unmarshal_msg_contact_tuples_from_json(&joined_json)?;

    let filtered_json = match (start, end) {
        (Some(start), Some(end)) => {
            apple_message_forensics::filter_date_between(joined, start, end)
        }
        (Some(start), None) => apple_message_forensics::filter_date_start(joined, start),
        (None, Some(end)) => apple_message_forensics::filter_date_end(joined, end),
        (None, None) => unreachable!("filter dates were checked before parsing joined messages"),
    };

    std::fs::write(json_dir.join("filtered.json"), filtered_json)?;
    Ok(true)
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Report) => {
            // Report only mode
            if !args.output.exists() {
                logger::error("Output directory does not exist. Please run collection first.");
                process::exit(1);
            }

            let joined_path = args.output.join("json").join("joined.json");
            if !joined_path.exists() {
                logger::error(
                    "Error: joined.json not found. Please collect and process evidence before running the report.",
                );
                process::exit(1);
            }

            logger::info_fmt(format_args!(
                "Generating report from existing collection at {}...",
                args.output.display()
            ));
            let filtering_applied =
                match filter_joined_messages(&args.output.join("json"), args.start, args.end) {
                    Ok(applied) => applied,
                    Err(err) => {
                        logger::error_fmt(format_args!("error: {err}"));
                        process::exit(1);
                    }
                };
            if let Err(err) = apple_message_forensics::report::generate(
                args.output.clone(),
                args.meta,
                filtering_applied,
            ) {
                logger::error_fmt(format_args!("error: {err}"));
                process::exit(1);
            }
            logger::info("Report generated successfully in output/report/index.html");
        }
        Some(Commands::Collect) | None => {
            // Standard collection and report flow
            let home_dir = default_home_dir();

            let chat_db_src_dir = home_dir.join("Library").join("Messages");
            let chat_db_init = chat_db_path_from_src_dir(&chat_db_src_dir);

            let chat_db = match check_source(chat_db_init) {
                Ok(path) => path,
                Err(err) => {
                    logger::error_fmt(format_args!("error: {err}"));
                    logger::error(
                        "hint: verify that the source is a valid iMessage chat database.",
                    );
                    process::exit(1);
                }
            };

            logger::info_fmt(format_args!(
                "source file is accessible: {}",
                chat_db.display()
            ));

            if let Err(err) = setup_output_dir(&args.output) {
                logger::error_fmt(format_args!("error: {err}"));
                logger::error("hint: specify a new or empty output directory with --output.");
                process::exit(1);
            }

            logger::init(&args.output).expect("to initialize logger");
            logger::info_fmt(format_args!(
                "output directory ready: {}",
                args.output.display()
            ));

            let file_hash_manifest = apple_message_forensics::write_evidence_hashes(
                home_dir.as_path(),
                args.output.as_path(),
            )
            .expect("file hash manifest to be created");
            let contents = std::fs::read_to_string(&file_hash_manifest).expect("to read manifest");
            logger::info_fmt(format_args!(
                "\nfound the following evidence files:\n{}",
                contents
            ));

            print!("Copying evidence files to output directory...");
            apple_message_forensics::copy_and_verify_manifest_files(
                file_hash_manifest.as_path(),
                home_dir.as_path(),
                args.output
                    .as_path()
                    .join("evidence")
                    .join("working_copy")
                    .as_path(),
            )
            .expect("evidence files to be copied");
            logger::info("done.");

            let vault_path = &args.output.join("evidence").join("vault").join("vault.zip");
            let vault_src = &args.output.join("evidence").join("working_copy");
            logger::info_fmt(format_args!(
                "archiving evidence from {} to {}",
                vault_src.as_path().display(),
                vault_path.as_path().display()
            ));
            apple_message_forensics::archive_evidence(vault_src.as_path(), vault_path.as_path())
                .expect("evidence archive to be created");
            logger::info_fmt(format_args!(
                "evidence archive created at {}",
                vault_path.as_path().display()
            ));

            let json_dir = args.output.join("json");
            let handles =
                apple_message_forensics::get_all_handles(&chat_db).expect("to get handles");
            logger::info_fmt(format_args!(
                "Extracting handles from chat database at {} ...",
                chat_db.display()
            ));
            let handles_path = json_dir.join("handles.json");
            std::fs::write(&handles_path, handles).expect("to write handles to JSON file");
            logger::info_fmt(format_args!(
                "Extracted handles to JSON file at {}",
                handles_path.display()
            ));

            logger::info("Extracting contacts from AddressBook-v22.abcddb ...");
            let addresses = apple_message_forensics::get_all_addresses(
                &file_hash_manifest,
                &args.output.join("evidence").join("working_copy"),
            )
            .expect("to get addresses");
            let addresses_path = json_dir.join("addresses.json");
            std::fs::write(&addresses_path, &addresses).expect("to write addresses to JSON file");
            logger::info_fmt(format_args!(
                "Extracted contacts to JSON file at {}",
                addresses_path.display()
            ));

            logger::info("Parsing contacts...");
            let contacts = apple_message_forensics::unmarshal_addresses_from_json(&addresses)
                .expect("to parse contacts");

            logger::info("Parsing messages...");
            let messages_json =
                apple_message_forensics::get_all_messages(&chat_db).expect("to get messages");
            let messages_path = json_dir.join("messages.json");
            std::fs::write(&messages_path, &messages_json).expect("to write messages to JSON file");

            let messages = apple_message_forensics::unmarshal_messages_from_json(&messages_json)
                .expect("to parse messages");

            logger::info("Joining messages with contacts...");
            let joined = apple_message_forensics::join_messages_to_contacts(&messages, &contacts)
                .expect("to join messages with contacts");

            logger::info("Marshaling joined messages and contacts to JSON...");
            let joined_json = apple_message_forensics::marshal_msg_contact_tuple_to_json(&joined);
            let joined_path = json_dir.join("joined.json");
            std::fs::write(&joined_path, joined_json.to_string())
                .expect("to write joined messages and contacts to JSON file");
            logger::info_fmt(format_args!(
                "Marshaled joined messages and contacts to JSON file at {}",
                joined_path.display()
            ));

            let filtering_applied = match filter_joined_messages(&json_dir, args.start, args.end) {
                Ok(applied) => applied,
                Err(err) => {
                    logger::error_fmt(format_args!("error: {err}"));
                    process::exit(1);
                }
            };

            // Final step: Generate report automatically after collection
            if let Err(err) = apple_message_forensics::report::generate(
                args.output.clone(),
                args.meta,
                filtering_applied,
            ) {
                logger::error_fmt(format_args!("error: {err}"));
                process::exit(1);
            }
            logger::info("Report generated successfully.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, Commands, filter_joined_messages};
    use chrono::{DateTime, Utc};
    use clap::Parser;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn date(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("valid test date")
            .with_timezone(&Utc)
    }

    fn json_dir() -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = tempdir().expect("create temporary directory");
        let json_dir = temp_dir.path().join("json");
        fs::create_dir(&json_dir).expect("create JSON output directory");
        fs::write(
            json_dir.join("joined.json"),
            json!([
                {"timestamp": "2024-01-01T00:00:00Z", "message": "before", "contact": null},
                {"timestamp": "2024-01-02T00:00:00Z", "message": "boundary", "contact": null},
                {"timestamp": "2024-01-03T00:00:00Z", "message": "after", "contact": null}
            ])
            .to_string(),
        )
        .expect("write joined messages");
        (temp_dir, json_dir)
    }

    fn filtered_messages(json_dir: &std::path::Path) -> Vec<String> {
        let json =
            fs::read_to_string(json_dir.join("filtered.json")).expect("read filtered messages");
        serde_json::from_str::<serde_json::Value>(&json)
            .expect("valid filtered JSON")
            .as_array()
            .expect("filtered JSON array")
            .iter()
            .map(|message| message["message"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn command_switches_parse_as_rfc3339_dates() {
        let args = Args::try_parse_from([
            "apple-message-forensics",
            "--start",
            "2024-01-01T00:00:00Z",
            "--end",
            "2024-01-03T00:00:00Z",
            "report",
        ])
        .expect("parse date switches");

        assert!(matches!(args.command, Some(Commands::Report)));
        assert_eq!(args.start, Some(date("2024-01-01T00:00:00Z")));
        assert_eq!(args.end, Some(date("2024-01-03T00:00:00Z")));
    }

    #[test]
    fn only_start_creates_filtered_json_from_start_date() {
        let (_temp_dir, json_dir) = json_dir();
        assert!(
            filter_joined_messages(&json_dir, Some(date("2024-01-02T00:00:00Z")), None,)
                .expect("filter joined messages")
        );
        assert_eq!(filtered_messages(&json_dir), ["boundary", "after"]);
    }

    #[test]
    fn only_end_creates_filtered_json_through_end_date() {
        let (_temp_dir, json_dir) = json_dir();
        assert!(
            filter_joined_messages(&json_dir, None, Some(date("2024-01-02T00:00:00Z")),)
                .expect("filter joined messages")
        );
        assert_eq!(filtered_messages(&json_dir), ["before", "boundary"]);
    }

    #[test]
    fn both_dates_create_filtered_json_including_bounds() {
        let (_temp_dir, json_dir) = json_dir();
        assert!(
            filter_joined_messages(
                &json_dir,
                Some(date("2024-01-02T00:00:00Z")),
                Some(date("2024-01-03T00:00:00Z")),
            )
            .expect("filter joined messages")
        );
        assert_eq!(filtered_messages(&json_dir), ["boundary", "after"]);
    }

    #[test]
    fn no_dates_does_not_create_filtered_json() {
        let (_temp_dir, json_dir) = json_dir();
        assert!(!filter_joined_messages(&json_dir, None, None).expect("skip filtering"));
        assert!(!json_dir.join("filtered.json").exists());
    }
}
