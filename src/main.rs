use clap::Parser;
use std::fs::{self, File};
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

fn check_source(chat_db: PathBuf) -> PathBuf {
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
        eprintln!("hint: grant the required access and try again.");
        eprintln!(
            "see: https://github.com/McFlip/apple-message-forensics#prerequisite-access"
        );
        process::exit(1);
    }

    chat_db
}

fn setup_output_dir(output: &PathBuf) {
    if output.exists() {
        if !output.is_dir() {
            eprintln!(
                "error: output path is not a directory: {}",
                output.display()
            );
            process::exit(1);
        }

        match fs::read_dir(output) {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    eprintln!(
                        "error: output directory is not empty: {}",
                        output.display()
                    );
                    eprintln!(
                        "hint: specify a new or empty output directory with --output."
                    );
                    process::exit(1);
                }
            }
            Err(err) => {
                eprintln!(
                    "error: cannot read output directory '{}': {}",
                    output.display(),
                    err
                );
                process::exit(1);
            }
        }
    } else if let Err(err) = fs::create_dir_all(output) {
        eprintln!(
            "error: cannot create output directory '{}': {}",
            output.display(),
            err
        );
        process::exit(1);
    }

    let directories = [
        output.join("evidence").join("vault"),
        output.join("evidence").join("working_copy"),
        output.join("json"),
        output.join("report"),
        output.join("logs"),
    ];

    if let Err(err) = directories
        .iter()
        .try_for_each(fs::create_dir_all)
    {
        eprintln!(
            "error: cannot create output directory structure under '{}': {}",
            output.display(),
            err
        );
        process::exit(1);
    }
}

fn main() {
    let args = Args::parse();

    let chat_db = match args.chat_db {
        Some(path) => path,
        None => default_chat_db(),
    };

    let chat_db = check_source(chat_db);

    println!("source file is accessible: {}", chat_db.display());

    setup_output_dir(&args.output);

    println!("output directory ready: {}", args.output.display());
}
