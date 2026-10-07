use chrono::Local;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(output_dir: &Path) -> Result<(), String> {
    let logs_dir = output_dir.join("logs");
    create_dir_all(&logs_dir).map_err(|e| format!("cannot create logs directory: {}", e))?;
    
    let log_file_path = logs_dir.join("logs.txt");
    let mut lock = LOG_FILE.lock().unwrap();
    *lock = Some(log_file_path);
    
    Ok(())
}

fn write_log(level: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let formatted = format!("[{}] [{}] {}\n", timestamp, level, message);
    
    print!("{}", formatted);

    if let Ok(lock) = LOG_FILE.lock()
        && let Some(path) = &*lock
        && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path)
    {
        let _ = file.write_all(formatted.as_bytes());
    }
}

pub fn info(message: &str) {
    write_log("INFO", message);
}

pub fn warn(message: &str) {
    write_log("WARN", message);
}

pub fn error(message: &str) {
    write_log("ERROR", message);
}

pub fn info_fmt(args: std::fmt::Arguments) {
    write_log("INFO", &args.to_string());
}

pub fn error_fmt(args: std::fmt::Arguments) {
    write_log("ERROR", &args.to_string());
}

pub fn warn_fmt(args: std::fmt::Arguments) {
    write_log("WARN", &args.to_string());
}
