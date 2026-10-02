use clap::Parser;
use std::fs::File;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the iMessage chat database
    chat_db: Option<PathBuf>,
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

    if !chat_db.exists() {
        eprintln!("error: source file does not exist: {}", chat_db.display());
        process::exit(1);
    }

    if !chat_db.is_file() {
        eprintln!(
            "error: source path is not a regular file: {}",
            chat_db.display()
        );
        process::exit(1);
    }

    if let Err(err) = File::open(&chat_db) {
        eprintln!(
            "error: cannot read source file '{}': {}",
            chat_db.display(),
            err
        );
        eprintln!(
            "error: cannot read source file '/Users/me/Library/Messages/chat.db': Permission denied\n
hint: grant the required access and try again.\n
see: https://github.com/McFlip/apple-message-forensics#prerequisite-access"
        );
        process::exit(1);
    }

    println!("source file is accessible: {}", chat_db.display());
}
