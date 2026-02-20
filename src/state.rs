use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExcludedEntry {
    pub path: String,
    pub pattern: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunState {
    pub last_run: String,
    pub excluded_count: usize,
    pub already_excluded_count: usize,
    pub entries: Vec<ExcludedEntry>,
}

fn state_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".local/state/tmignore")
}

fn state_path() -> PathBuf {
    state_dir().join("state.json")
}

pub fn save_state(state: &RunState) -> Result<()> {
    std::fs::create_dir_all(state_dir()).context("Failed to create state directory")?;
    let contents = serde_json::to_string_pretty(state).context("Failed to serialize state")?;
    std::fs::write(state_path(), contents).context("Failed to write state file")?;
    Ok(())
}

pub fn load_state() -> Result<Option<RunState>> {
    let path = state_path();
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let state: RunState = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    Ok(Some(state))
}
