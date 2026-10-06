use apple_message_forensics::check_source;
use apple_message_forensics::setup_output_dir;
use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the folder containing the iMessage chat database and Address Books
    home_dir: Option<PathBuf>,

    /// Path to the output directory
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

fn default_home_dir() -> PathBuf {
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home),
        None => {
            eprintln!("error: HOME environment variable is not set");
            process::exit(1);
        }
    }
}

fn chat_db_path_from_src_dir(src_dir: &PathBuf) -> PathBuf {
    src_dir.join("chat.db")
}

fn main() {
    let args = Args::parse();

    let home_dir = match args.home_dir {
        Some(path) => path,
        None => default_home_dir(),
    };

    let chat_db_src_dir = home_dir.join("Library").join("Messages");

    let chat_db = chat_db_path_from_src_dir(&chat_db_src_dir);

    let chat_db = match check_source(chat_db) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!("hint: verify that the source is a valid iMessage chat database.");
            process::exit(1);
        }
    };

    println!("source file is accessible: {}", chat_db.display());

    if let Err(err) = setup_output_dir(&args.output) {
        eprintln!("error: {err}");
        eprintln!("hint: specify a new or empty output directory with --output.");
        process::exit(1);
    }

    println!("output directory ready: {}", args.output.display());

    let file_hash_manifest =
        apple_message_forensics::write_evidence_hashes(home_dir.as_path(), args.output.as_path())
            .expect("file hash manifest to be created");
    // Read and print the contents of the manifest file
    let contents = std::fs::read_to_string(&file_hash_manifest).expect("to read manifest");
    print!("\nfound the following evidence files:\n{}", contents);

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
    println!("done.");

    // zip working directory to vault
    let vault_path = &args.output.join("evidence").join("vault").join("vault.zip");
    let vault_src = &args.output.join("evidence").join("working_copy");
    println!(
        "archiving evidence from {} to {}",
        vault_src.as_path().display(),
        vault_path.as_path().display()
    );
    apple_message_forensics::archive_evidence(vault_src, vault_path)
        .expect("evidence archive to be created");
    println!(
        "evidence archive created at {}",
        vault_path.as_path().display()
    );

    // TODO: parse chat.db and AddressBook-v22.abcddb to extract messages and contacts to JSON
    let json_dir = args.output.join("json");
    let handles = apple_message_forensics::get_all_handles(&chat_db).expect("to get handles");
    println!(
        "Extracting handles from chat database at {} ...",
        chat_db.display()
    );
    let handles_path = json_dir.join("handles.json");
    std::fs::write(&handles_path, handles).expect("to write handles to JSON file");
    println!(
        "Extracted handles to JSON file at {}",
        handles_path.display()
    );

    // TODO: Create HTML report from JSON data and attachments
    // TODO: Create deliverable package with report, JSON, evidence, and manifest
}
