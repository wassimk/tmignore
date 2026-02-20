use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

const LABEL: &str = "com.wassimk.tmignore";

fn plist_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Library/LaunchAgents/com.wassimk.tmignore.plist")
}

fn log_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Library/Logs/tmignore")
}

fn current_uid() -> String {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .expect("failed to run id -u");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn generate_plist(binary_path: &str) -> String {
    let log_dir = log_dir();
    let stdout_log = log_dir.join("stdout.log");
    let stderr_log = log_dir.join("stderr.log");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary_path}</string>
        <string>run</string>
    </array>
    <key>StartInterval</key>
    <integer>86400</integer>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>"#,
        stdout = stdout_log.display(),
        stderr = stderr_log.display(),
    )
}

pub fn install(force: bool) -> Result<()> {
    let plist = plist_path();

    if plist.exists() && !force {
        anyhow::bail!(
            "LaunchAgent already installed at {}\nUse --force to overwrite.",
            plist.display()
        );
    }

    let binary_path = std::env::current_exe()
        .context("Failed to determine binary path")?
        .to_string_lossy()
        .to_string();

    // Unload existing agent if overwriting
    if plist.exists() {
        let _ = Command::new("launchctl")
            .args(["bootout", &format!("gui/{}/{LABEL}", current_uid())])
            .output();
    }

    // Create log directory
    std::fs::create_dir_all(log_dir()).context("Failed to create log directory")?;

    // Write plist
    let content = generate_plist(&binary_path);
    std::fs::write(&plist, content)
        .with_context(|| format!("Failed to write plist to {}", plist.display()))?;

    // Load agent
    let output = Command::new("launchctl")
        .args([
            "bootstrap",
            &format!("gui/{}", current_uid()),
            &plist.to_string_lossy(),
        ])
        .output()
        .context("Failed to run launchctl bootstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("launchctl bootstrap failed: {}", stderr.trim());
    }

    println!("LaunchAgent installed and loaded.");
    println!("  Label: {LABEL}");
    println!("  Plist: {}", plist.display());
    println!("  Logs:  {}", log_dir().display());
    println!();
    println!("The service will run `tmignore run` every 24 hours.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    // Unload (ignore errors if not loaded)
    let _ = Command::new("launchctl")
        .args(["bootout", &format!("gui/{}/{LABEL}", current_uid())])
        .output();

    let plist = plist_path();
    if plist.exists() {
        std::fs::remove_file(&plist)
            .with_context(|| format!("Failed to remove {}", plist.display()))?;
        println!("LaunchAgent uninstalled.");
    } else {
        println!("LaunchAgent was not installed.");
    }

    Ok(())
}

pub fn status() -> Result<(bool, bool)> {
    let output = Command::new("launchctl")
        .args(["list", LABEL])
        .output()
        .context("Failed to run launchctl list")?;

    let running = output.status.success();
    let installed = plist_path().exists();

    Ok((installed, running))
}

pub fn label() -> &'static str {
    LABEL
}

pub fn get_plist_path() -> PathBuf {
    plist_path()
}

pub fn get_log_dir() -> PathBuf {
    log_dir()
}
