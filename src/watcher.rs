use anyhow::{Context, Result};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;

/// Struct representing the file watcher
pub struct FileWatcher {
    watcher: RecommendedWatcher,
}

impl FileWatcher {
    /// Creates a new instance of `FileWatcher`
    ///
    /// # Arguments
    /// - `tx`: A channel sender for receiving file events
    ///
    /// # Returns
    /// - `anyhow::Result<Self>`: The constructed `FileWatcher` instance or an error
    pub fn new(tx: Sender<notify::Result<Event>>) -> Result<Self> {
        let watcher = RecommendedWatcher::new(tx, Config::default())
            .context("Failed to create file watcher")?;
        Ok(Self { watcher })
    }

    /// Starts watching the specified path
    ///
    /// # Arguments
    /// - `path`: The path to watch
    ///
    /// # Returns
    /// - `anyhow::Result<()>`: `Ok` if watching starts successfully, otherwise an error
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .context(format!("Failed to watch path: {}", path.display()))?;
        Ok(())
    }
}

