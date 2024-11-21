use anyhow::{Context, Result};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(tx: Sender<notify::Result<Event>>) -> Result<Self> {
        let watcher = RecommendedWatcher::new(tx, Config::default())
            .context("Failed to create file watcher")?;
        Ok(Self { watcher })
    }

    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .context(format!("Failed to watch path: {}", path.display()))?;
        Ok(())
    }
}
