use apple_message_forensics::check_source;
use apple_message_forensics::setup_output_dir;
use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the iMessage chat database
    chat_db: Option<PathBuf>,

    /// Path to the output directory
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

fn default_chat_db() -> PathBuf {
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home)
            .join("Library")
            .join("Messages")
            .join("chat.db"),
        None => {
            eprintln!("error: HOME environment variable is not set");
            process::exit(1);
        }
    }
}

fn main() {
    let args = Args::parse();

    let chat_db = match args.chat_db {
        Some(path) => path,
        None => default_chat_db(),
    };

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
}
