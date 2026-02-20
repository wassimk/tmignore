use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Check if a path is already excluded from Time Machine backups.
pub fn is_excluded(path: &Path) -> Result<bool> {
    let output = Command::new("tmutil")
        .args(["isexcluded", &path.to_string_lossy()])
        .output()
        .with_context(|| format!("Failed to run tmutil isexcluded on {}", path.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // tmutil isexcluded outputs "[Excluded] <path>" or "[Included] <path>"
    Ok(stdout.contains("[Excluded]"))
}

/// Add a sticky exclusion to a path (writes extended attribute, no root needed).
pub fn add_exclusion(path: &Path) -> Result<()> {
    let output = Command::new("tmutil")
        .args(["addexclusion", &path.to_string_lossy()])
        .output()
        .with_context(|| format!("Failed to run tmutil addexclusion on {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmutil addexclusion failed for {}: {}", path.display(), stderr.trim());
    }

    Ok(())
}

/// Remove a sticky exclusion from a path.
pub fn remove_exclusion(path: &Path) -> Result<()> {
    let output = Command::new("tmutil")
        .args(["removeexclusion", &path.to_string_lossy()])
        .output()
        .with_context(|| format!("Failed to run tmutil removeexclusion on {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "tmutil removeexclusion failed for {}: {}",
            path.display(),
            stderr.trim()
        );
    }

    Ok(())
}

/// Get the size of a directory using `du -sh`.
pub fn dir_size(path: &Path) -> String {
    Command::new("du")
        .args(["-sh", &path.to_string_lossy()])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                s.split_whitespace().next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "?".to_string())
}
