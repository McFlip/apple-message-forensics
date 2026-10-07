use apple_message_forensics::check_source;
use apple_message_forensics::setup_output_dir;
use apple_message_forensics::logger;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
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

fn chat_db_path_from_src_dir(src_dir: &PathBuf) -> PathBuf {
    src_dir.join("chat.db")
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
                logger::error("Error: joined.json not found. Please collect and process evidence before running the report.");
                process::exit(1);
            }

            logger::info_fmt(format_args!("Generating report from existing collection at {}...", args.output.display()));
            if let Err(err) = apple_message_forensics::report::generate(args.output.clone(), args.meta) {
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
                    logger::error("hint: verify that the source is a valid iMessage chat database.");
                    process::exit(1);
                }
            };

            logger::info_fmt(format_args!("source file is accessible: {}", chat_db.display()));

            if let Err(err) = setup_output_dir(&args.output) {
                logger::error_fmt(format_args!("error: {err}"));
                logger::error("hint: specify a new or empty output directory with --output.");
                process::exit(1);
            }

            logger::init(&args.output).expect("to initialize logger");
            logger::info_fmt(format_args!("output directory ready: {}", args.output.display()));

            let file_hash_manifest =
                apple_message_forensics::write_evidence_hashes(home_dir.as_path(), args.output.as_path())
                    .expect("file hash manifest to be created");
            let contents = std::fs::read_to_string(&file_hash_manifest).expect("to read manifest");
            logger::info_fmt(format_args!("\nfound the following evidence files:\n{}", contents));

            print!("Copying evidence files to output directory...");
            apple_message_forensics::copy_and_verify_manifest_files(
                file_hash_manifest.as_path(),
                home_dir.as_path(),
                args.output.as_path().join("evidence").join("working_copy").as_path(),
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
            logger::info_fmt(format_args!(
                "evidence archive created at {}",
                vault_path.as_path().display()
            ));

            let json_dir = args.output.join("json");
            let handles = apple_message_forensics::get_all_handles(&chat_db).expect("to get handles");
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
            let messages_json = apple_message_forensics::get_all_messages(&chat_db).expect("to get messages");
            let messages_path = json_dir.join("messages.json");
            std::fs::write(&messages_path, &messages_json).expect("to write messages to JSON file");

            let messages = apple_message_forensics::unmarshal_messages_from_json(&messages_json)
                .expect("to parse messages");

            logger::info("Joining messages with contacts...");
            let joined = apple_message_forensics::join_messages_to_contacts(&messages, &contacts);

            logger::info("Marshaling joined messages and contacts to JSON...");
            let joined_json = apple_message_forensics::marshal_msg_contact_tuple_to_json(&joined);
            let joined_path = json_dir.join("joined.json");
            std::fs::write(&joined_path, &joined_json.to_string())
                .expect("to write joined messages and contacts to JSON file");
            logger::info_fmt(format_args!(
                "Marshaled joined messages and contacts to JSON file at {}",
                joined_path.display()
            ));

            // Final step: Generate report automatically after collection
            if let Err(err) = apple_message_forensics::report::generate(args.output.clone(), args.meta) {
                logger::error_fmt(format_args!("error: {err}"));
                process::exit(1);
            }
            logger::info("Report generated successfully.");
        }
    }
}
