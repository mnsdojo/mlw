use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{mpsc::channel, Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use clap::Parser;
use logger::{log, verbose_log, LogLevel};
use notify::EventKind;
use regex::Regex;
use serde::Deserialize;
use watcher::FileWatcher;

mod logger;
mod watcher;

#[derive(Parser, Debug)]
#[command(
    name = "mlw",
    about = "A file watcher for multi languages",
    version,
    author
)]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "mlw.toml")]
    config: String,

    /// Generate a default config file
    #[arg(long, short)]
    gen_config: bool,
}

#[derive(Deserialize, Clone, Debug)]
struct ConfigFile {
    path: Vec<String>,
    script_args: Option<Vec<String>>, // Added to support additional arguments
    delay: u64,
    verbose: Option<bool>,
    ignore_pattern: Option<String>,
    script_type: Option<String>,
}

struct ScriptProcess {
    child: Option<Child>,
}

const DEFAULT_CONFIG: &str = r#"
# Default mlw configuration file
# Path(s) to watch
path = ["./src"]

# Delay (in seconds) between script restarts
delay = 2

# Verbose logging
verbose = true

# Pattern for files to ignore (optional)
ignore_pattern = ".*\\.git.*"

# Type of script to run (e.g. python, node, go)
script_type = "node"


# Additional arguments for the script (optional)
# script_args = ["--dev", "--watch"]
"#;

impl ScriptProcess {
    fn new() -> Self {
        Self { child: None }
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn get_command_config(script_type: &str) -> Result<(&'static str, Vec<&'static str>)> {
        match script_type {
            // Interpreted languages
            "python" => Ok(("python3", vec![])),
            "python2" => Ok(("python2", vec![])),
            "node" => Ok(("node", vec![])),
            "lua" => Ok(("lua", vec![])),
            "php" => Ok(("php", vec![])),

            // Compiled languages
            "go" => Ok(("go", vec!["run"])),
            "rust" => Ok(("cargo", vec!["run", "--"])),

            // shell
            "sh" => Ok(("sh", vec![])),

            unknown => anyhow::bail!("Unsupported script type: {}", unknown),
        }
    }

    fn restart(&mut self, config: &ConfigFile) -> Result<()> {
        self.stop();

        let script_type = config
            .script_type
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Missing script type in config"))?;

        let (command, default_args) = Self::get_command_config(script_type)?;

        verbose_log(
            LogLevel::Info,
            &format!("Restarting script using: {}", command),
            config.verbose,
        );

        for path in &config.path {
            // Combine default arguments with user-provided arguments
            let mut args = default_args.to_vec();
            args.push(path.as_str());

            // Add any additional arguments from config
            if let Some(extra_args) = &config.script_args {
                args.extend(extra_args.iter().map(String::as_str));
            }

            verbose_log(
                LogLevel::Debug,
                &format!("Running command: {} with args: {:?}", command, args),
                config.verbose,
            );

            let child = Command::new(command)
                .args(&args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .with_context(|| format!("Failed to start {} script", script_type))?;

            self.child = Some(child);
        }
        Ok(())
    }
}

fn load_config(file_path: &Path) -> Result<ConfigFile> {
    let config_str = fs::read_to_string(file_path).context("Failed to read config file")?;
    let config: ConfigFile = toml::from_str(&config_str).context("Failed to parse config file")?;

    // Check if any paths exist
    if config.path.is_empty() || !config.path.iter().all(|p| Path::new(p).exists()) {
        anyhow::bail!("One or more specified paths do not exist");
    }

    Ok(config)
}

fn handle_change(config: &ConfigFile, script_process: &mut ScriptProcess) -> Result<()> {
    verbose_log(
        LogLevel::Info,
        "File change detected. Restarting...",
        config.verbose,
    );
    std::thread::sleep(Duration::from_secs(config.delay));
    script_process.restart(config)?;
    verbose_log(
        LogLevel::Info,
        "script restarted successfully.",
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

fn generate_default_config(output_path: &Path) -> Result<()> {
    if output_path.exists() {
        anyhow::bail!("Config file already exists at {:?}", output_path);
    }

    fs::write(output_path, DEFAULT_CONFIG).context("Failed to write config file")?;
    println!("Default configuration file generated at {:?}", output_path);

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Generate default config if the flag is set
    if cli.gen_config {
        let config_path = Path::new(&cli.config);
        generate_default_config(config_path)?;
        return Ok(());
    }

    let config = load_config(Path::new(&cli.config))?;

    if config.verbose.unwrap_or(false) {
        log(LogLevel::Info, "Configuration loaded.");
    }

    let (tx, rx) = channel();
    let mut file_watcher = FileWatcher::new(tx)?;
    for path in &config.path {
        file_watcher.watch(Path::new(path))?;
    }

    let mut script_process = ScriptProcess::new();
    script_process.restart(&config)?;

    if config.verbose.unwrap_or(false) {
        for path in &config.path {
            log(LogLevel::Info, &format!("Watching path: {}", path));
        }
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

                            if let Err(e) = handle_change(&config, &mut script_process) {
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
