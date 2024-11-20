use std::{path::Path, sync::mpsc::Sender};

use anyhow::Context;
use notify::{Config, Event, RecommendedWatcher, Watcher};

mod logger;
mod watcher;
fn main() {}

pub struct FileWatcher {
    watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(tx: Sender<notify::Result<Event>>) -> anyhow::Result<Self> {
        let watcher = RecommendedWatcher::new(tx, Config::default())
            .context("Failed to create file watcher")?;
        Ok(Self { watcher })
    }

    pub fn watch(&mut self, path: &Path) -> anyhow::Result<()> {
        self.watcher
            .watch(path, notify::RecursiveMode::Recursive)
            .context(format!("Faileed to match path : {}", path.display()))?;
        Ok(())
    }
    
}
