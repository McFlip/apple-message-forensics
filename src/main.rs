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
        eprintln!("see: https://github.com/McFlip/apple-message-forensics#prerequisite-access");
        process::exit(1);
    }

    chat_db
}

fn setup_output_dir(output: &PathBuf) -> Result<(), String> {
    if output.exists() {
        if !output.is_dir() {
            return Err(format!(
                "output path is not a directory: {}",
                output.display()
            ));
        }

        let mut entries = fs::read_dir(output).map_err(|err| {
            format!(
                "cannot read output directory '{}': {}",
                output.display(),
                err
            )
        })?;

        if entries.next().is_some() {
            return Err(format!(
                "output directory is not empty: {}",
                output.display()
            ));
        }
    } else {
        fs::create_dir_all(output).map_err(|err| {
            format!(
                "cannot create output directory '{}': {}",
                output.display(),
                err
            )
        })?;
    }

    let directories = [
        output.join("evidence").join("vault"),
        output.join("evidence").join("working_copy"),
        output.join("json"),
        output.join("report"),
        output.join("logs"),
    ];

    for directory in directories {
        fs::create_dir_all(&directory).map_err(|err| {
            format!(
                "cannot create output directory '{}': {}",
                directory.display(),
                err
            )
        })?;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    let chat_db = match args.chat_db {
        Some(path) => path,
        None => default_chat_db(),
    };

    let chat_db = check_source(chat_db);

    println!("source file is accessible: {}", chat_db.display());

    if let Err(err) = setup_output_dir(&args.output) {
        eprintln!("error: {err}");
        eprintln!("hint: specify a new or empty output directory with --output.");
        process::exit(1);
    }

    println!("output directory ready: {}", args.output.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn setup_output_dir_creates_missing_directory() {
        let temp = tempdir().expect("failed to create temporary directory");
        let output = temp.path().join("output");

        assert!(!output.exists());

        setup_output_dir(&output).unwrap();

        assert!(output.is_dir());
        assert!(output.join("evidence").join("vault").is_dir());
        assert!(output.join("evidence").join("working_copy").is_dir());
        assert!(output.join("json").is_dir());
        assert!(output.join("report").is_dir());
        assert!(output.join("logs").is_dir());
    }

    #[test]
    fn setup_output_dir_accepts_existing_empty_directory() {
        let temp = tempdir().expect("failed to create temporary directory");
        let output = temp.path().join("output");

        fs::create_dir(&output).expect("failed to create output directory");

        assert!(output.is_dir());
        assert_eq!(
            fs::read_dir(&output)
                .expect("failed to read output directory")
                .count(),
            0
        );

        setup_output_dir(&output).unwrap();

        assert!(output.join("evidence").join("vault").is_dir());
        assert!(output.join("evidence").join("working_copy").is_dir());
        assert!(output.join("json").is_dir());
        assert!(output.join("report").is_dir());
        assert!(output.join("logs").is_dir());
    }
    #[test]
    fn setup_output_dir_rejects_non_empty_directory() {
        let temp = tempdir().expect("failed to create temporary directory");
        let output = temp.path().join("output");

        fs::create_dir(&output).expect("failed to create output directory");
        fs::write(output.join("existing.txt"), "test").expect("failed to create existing file");

        let result = setup_output_dir(&output);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("output directory is not empty: {}", output.display())
        );
    }

    #[test]
    fn setup_output_dir_rejects_file_as_output_path() {
        let temp = tempdir().expect("failed to create temporary directory");
        let output = temp.path().join("output");

        fs::write(&output, "test").expect("failed to create test file");

        let result = setup_output_dir(&output);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("output path is not a directory: {}", output.display())
        );
    }
}
