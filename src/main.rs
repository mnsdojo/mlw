use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{mpsc::channel, Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use logger::{log, verbose_log, LogLevel};
use notify::EventKind;
use regex::Regex;
use serde::Deserialize;
use watcher::FileWatcher;

mod logger;
mod watcher;

#[derive(Deserialize, Clone, Debug)]
struct ConfigFile {
    path: String,
    delay: u64,
    verbose: Option<bool>,
    ignore_pattern: Option<String>,
    python_interpreter: Option<String>,
}

struct PythonProcess {
    child: Option<Child>,
}

impl PythonProcess {
    fn new() -> Self {
        Self { child: None }
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn restart(&mut self, config: &ConfigFile) -> Result<()> {
        self.stop();
        let interpreter = config.python_interpreter.as_deref().unwrap_or("python3");

        verbose_log(
            LogLevel::Info,
            &format!(
                "Restarting Python script using interpreter: {}",
                interpreter
            ),
            config.verbose,
        );

        let child = Command::new(interpreter)
            .arg(&config.path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to start Python script")?;

        self.child = Some(child);
        Ok(())
    }
}
fn load_config(file_path: &Path) -> Result<ConfigFile> {
    let config_str = fs::read_to_string(file_path).context("Failed to read config file")?;
    let config: ConfigFile = toml::from_str(&config_str).context("Failed to parse config file")?;
    if !Path::new(&config.path).exists() {
        anyhow::bail!("Specified path does not exist: {}", config.path);
    }

    Ok(config)
}
fn handle_change(config: &ConfigFile, python_process: &mut PythonProcess) -> Result<()> {
    verbose_log(
        LogLevel::Info,
        "File change detected. Restarting...",
        config.verbose,
    );
    std::thread::sleep(Duration::from_secs(config.delay));
    python_process.restart(config)?;
    verbose_log(
        LogLevel::Info,
        "Python script restarted successfully.",
        config.verbose,
    );
    Ok(())
}

fn should_ignore_path(path: &Path, ignore_pattern: Option<&str>) -> bool {
    ignore_pattern
        .and_then(|pattern| Regex::new(pattern).ok())
        .map(|regex| regex.is_match(&path.to_string_lossy()))
        .unwrap_or(false)
}

fn main() -> Result<()> {
    let config = load_config(Path::new("pew.toml"))?;

    if config.verbose.unwrap_or(false) {
        log(LogLevel::Info, "Configuration loaded.");
    }

    let (tx, rx) = channel();
    let mut file_watcher = FileWatcher::new(tx)?;
    file_watcher.watch(Path::new(&config.path))?;

    let mut python_process = PythonProcess::new();
    python_process.restart(&config)?;

    if config.verbose.unwrap_or(false) {
        log(LogLevel::Info, &format!("Watching path: {}", config.path));
    }
    let last_event_time = Arc::new(Mutex::new(Instant::now()));
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let Some(path) = event.paths.first() {
                    if should_ignore_path(path, config.ignore_pattern.as_deref()) {
                        if config.verbose.unwrap_or(false) {
                            log(LogLevel::Debug, &format!("Ignored file: {:?}", path));
                        }
                        continue;
                    }

                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) {
                        let now = Instant::now();
                        let mut last_event_time = last_event_time.lock().unwrap();

                        if now.duration_since(*last_event_time) > Duration::from_secs(config.delay)
                        {
                            *last_event_time = now; // Update the last event time

                            if let Err(e) = handle_change(&config, &mut python_process) {
                                log(LogLevel::Error, &format!("Error handling change: {}", e));
                            }
                        } else if config.verbose.unwrap_or(false) {
                            log(LogLevel::Debug, "Ignoring event due to debounce");
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                verbose_log(
                    LogLevel::Error,
                    &format!("Change handling error: {}", e),
                    config.verbose,
                );
            }
            Err(e) => {
                verbose_log(
                    LogLevel::Error,
                    &format!("Failed to receive file event: {}", e),
                    config.verbose,
                );
                break;
            }
        }
    }

    Ok(())
}
