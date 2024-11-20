use anyhow::{Context, Result};
use colored::*;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};


