use colored::*;

/// Enum representing the log levels

pub enum LogLevel {
    Info,
    Error,
    Debug,
}

pub fn verbose_log(level: LogLevel, message: &str, verbose: Option<bool>) {
    match verbose {
        Some(true) => {
            log(level, message);
        }
        _ => {
            match level {
                LogLevel::Info | LogLevel::Error => log(level, message),
                _ => {} // Skip Debug level logs if not verbose
            }
        }
    }
}
/// Logs messages to the console
pub fn log(level: LogLevel, message: &str) {
    match level {
        LogLevel::Info => println!("{}", format!("[INFO] {}", message).green()),
        LogLevel::Error => eprintln!("{}", format!("[ERROR] {}", message).red()),
        LogLevel::Debug => println!("{}", format!("[DEBUG] {}", message).yellow()),
    }
}
