use std::fmt::format;

use colored::*;

/// Enum representing the log levels

pub enum LogLevel {
    Info,
    Error,
    Debug,
}

/// Logs messages to the console
pub fn log(level: LogLevel, message: &str) {
    match level {
        LogLevel::Info => println!("{}", format!("[INFO] {}", message).green()),
        LogLevel::Error => eprintln!("{}", format!("[ERROR] {}", message).red()),
        LogLevel::Debug => println!("{}", format!("[DEBUG] {}", message).yellow()),
    }
}
