mod config;
mod excluder;
mod patterns;
mod scanner;
mod service;
mod state;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::{contract_tilde, expand_tilde};
use state::{ExcludedEntry, RunState};


#[derive(Parser, Debug)]
#[command(
    name = "tmignore",
    about = "Exclude developer dependency directories and arbitrary paths from macOS backups",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Scan for dependency directories and exclude them from backups
    Run {
        /// Show what would be excluded without making changes
        #[arg(long)]
        dry_run: bool,

        /// Print detailed output during scanning
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show currently excluded paths from the last run
    List,

    /// Add an arbitrary path to config and exclude it immediately
    Add {
        /// Path to exclude (supports ~ expansion)
        path: String,
    },

    /// Remove a path from config and un-exclude it
    Remove {
        /// Path to un-exclude (supports ~ expansion)
        path: String,
    },

    /// Show service status and last run statistics
    Status,

    /// Generate a default config file
    Init {
        /// Overwrite existing config file
        #[arg(long)]
        overwrite: bool,
    },

    /// Install the LaunchAgent for automatic background runs
    Install {
        /// Overwrite existing LaunchAgent
        #[arg(short, long)]
        force: bool,
    },

    /// Remove the LaunchAgent
    Uninstall,

    /// Remove backup exclusions set by tmignore
    Reset {
        /// Also remove ALL sticky exclusions on the system, including those set outside tmignore
        #[arg(long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Run { dry_run, verbose } => cmd_run(dry_run, verbose),
        Cmd::List => cmd_list(),
        Cmd::Add { path } => cmd_add(&path),
        Cmd::Remove { path } => cmd_remove(&path),
        Cmd::Status => cmd_status(),
        Cmd::Init { overwrite } => cmd_init(overwrite),
        Cmd::Install { force } => service::install(force),
        Cmd::Uninstall => service::uninstall(),
        Cmd::Reset { all } => cmd_reset(all),
    }
}

fn cmd_run(dry_run: bool, verbose: bool) -> Result<()> {
    let config = config::load_config()?;
    let active_patterns = patterns::resolve_patterns(&config.disable_patterns, &config.custom_patterns);

    if verbose {
        println!(
            "Scanning with {} active patterns across {} root(s)...",
            active_patterns.len(),
            config.scan_roots.len()
        );
    }

    let matches = scanner::scan_optimized(&config, &active_patterns);

    if verbose {
        println!("Found {} candidate directories.", matches.len());
    }

    let mut newly_excluded: Vec<ExcludedEntry> = Vec::new();
    let mut already_excluded_count: usize = 0;
    let mut error_count: usize = 0;

    for m in &matches {
        match excluder::is_excluded(&m.path) {
            Ok(true) => {
                already_excluded_count += 1;
                if verbose {
                    println!(
                        "  [skip] {} (already excluded)",
                        contract_tilde(&m.path.to_string_lossy())
                    );
                }
            }
            Ok(false) => {
                let display_path = contract_tilde(&m.path.to_string_lossy());

                if dry_run {
                    let size = excluder::dir_size(&m.path);
                    println!("  [dry-run] {} ({}, {})", display_path, m.pattern_name, size);
                    newly_excluded.push(ExcludedEntry {
                        path: display_path,
                        pattern: m.pattern_name.clone(),
                        size,
                    });
                } else {
                    match excluder::add_exclusion(&m.path) {
                        Ok(()) => {
                            let size = excluder::dir_size(&m.path);
                            println!("  [excluded] {} ({}, {})", display_path, m.pattern_name, size);
                            newly_excluded.push(ExcludedEntry {
                                path: display_path,
                                pattern: m.pattern_name.clone(),
                                size,
                            });
                        }
                        Err(e) => {
                            eprintln!("  [error] {}: {}", display_path, e);
                            error_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "  [error] checking {}: {}",
                    contract_tilde(&m.path.to_string_lossy()),
                    e
                );
                error_count += 1;
            }
        }
    }

    // Print summary
    println!();
    if dry_run {
        println!("Dry run complete.");
    }
    println!(
        "  {} newly excluded, {} already excluded, {} errors",
        newly_excluded.len(),
        already_excluded_count,
        error_count
    );

    // Save state (even for dry-run, to record what was found)
    if !dry_run {
        let run_state = RunState {
            last_run: chrono_now(),
            excluded_count: newly_excluded.len(),
            already_excluded_count,
            entries: newly_excluded,
        };
        state::save_state(&run_state)?;
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    match state::load_state()? {
        Some(run_state) => {
            if run_state.entries.is_empty() {
                println!("No paths were excluded in the last run.");
            } else {
                println!("Paths excluded in last run ({}):", run_state.last_run);
                println!();
                for entry in &run_state.entries {
                    println!("  {} ({}, {})", entry.path, entry.pattern, entry.size);
                }
                println!();
                println!(
                    "  {} excluded, {} already excluded",
                    run_state.excluded_count, run_state.already_excluded_count
                );
            }
        }
        None => {
            println!("No previous run found. Run `tmignore run` first.");
        }
    }
    Ok(())
}

fn cmd_add(path_str: &str) -> Result<()> {
    let expanded = expand_tilde(path_str);
    let canonical = if expanded.exists() {
        expanded
            .canonicalize()
            .with_context(|| format!("Failed to resolve path: {}", expanded.display()))?
    } else {
        anyhow::bail!("Path does not exist: {}", expanded.display());
    };

    // Add to config
    let mut cfg = config::load_config()?;
    let tilde_path = contract_tilde(&canonical.to_string_lossy());

    if cfg.extra_exclude_paths.contains(&tilde_path) {
        println!("{} is already in exclude_paths.", tilde_path);
    } else {
        cfg.extra_exclude_paths.push(tilde_path.clone());
        config::save_config(&cfg)?;
        println!("Added {} to config.", tilde_path);
    }

    // Exclude immediately
    if excluder::is_excluded(&canonical)? {
        println!("{} is already excluded from backups.", tilde_path);
    } else {
        excluder::add_exclusion(&canonical)?;
        println!("Excluded {} from backups.", tilde_path);
    }

    Ok(())
}

fn cmd_remove(path_str: &str) -> Result<()> {
    let expanded = expand_tilde(path_str);
    let canonical = if expanded.exists() {
        expanded
            .canonicalize()
            .with_context(|| format!("Failed to resolve path: {}", expanded.display()))?
    } else {
        // Path might not exist anymore, but still try to remove from config
        expanded
    };

    // Remove from config
    let mut cfg = config::load_config()?;
    let tilde_path = contract_tilde(&canonical.to_string_lossy());
    let original_len = cfg.extra_exclude_paths.len();
    cfg.extra_exclude_paths.retain(|p| p != &tilde_path);

    if cfg.extra_exclude_paths.len() < original_len {
        config::save_config(&cfg)?;
        println!("Removed {} from config.", tilde_path);
    } else {
        println!("{} was not in exclude_paths.", tilde_path);
    }

    // Un-exclude
    if canonical.exists() {
        if excluder::is_excluded(&canonical)? {
            excluder::remove_exclusion(&canonical)?;
            println!("Removed backup exclusion for {}.", tilde_path);
        } else {
            println!("{} was not excluded from backups.", tilde_path);
        }
    }

    Ok(())
}

fn cmd_status() -> Result<()> {
    let (installed, running) = service::status()?;

    println!("Service:     {}", service::label());
    println!("Installed:   {}", if installed { "yes" } else { "no" });
    println!("Running:     {}", if running { "yes" } else { "no" });
    println!();

    // Show last run info
    match state::load_state()? {
        Some(run_state) => {
            println!("Last run:    {}", run_state.last_run);
            println!(
                "  {} excluded, {} already excluded",
                run_state.excluded_count, run_state.already_excluded_count
            );
        }
        None => {
            println!("Last run:    never");
        }
    }

    println!();
    println!("Paths:");
    println!(
        "  Config: {}",
        contract_tilde(&config::config_path().to_string_lossy())
    );
    println!(
        "  Plist:  {}",
        contract_tilde(&service::get_plist_path().to_string_lossy())
    );
    println!(
        "  Logs:   {}",
        contract_tilde(&service::get_log_dir().to_string_lossy())
    );

    Ok(())
}

fn cmd_init(overwrite: bool) -> Result<()> {
    let path = config::config_path();

    if path.exists() && !overwrite {
        anyhow::bail!(
            "Config already exists at {}\nUse --overwrite to replace it.",
            path.display()
        );
    }

    std::fs::create_dir_all(config::config_dir()).context("Failed to create config directory")?;
    std::fs::write(&path, config::Config::default_toml())
        .with_context(|| format!("Failed to write {}", path.display()))?;

    println!("Created default config at {}", contract_tilde(&path.to_string_lossy()));
    Ok(())
}

fn cmd_reset(all: bool) -> Result<()> {
    let mut removed_count: usize = 0;
    let mut error_count: usize = 0;

    if all {
        // Find ALL sticky exclusions on the system using mdfind
        println!("Finding all sticky backup exclusions on the system...");
        let output = std::process::Command::new("mdfind")
            .args(["com_apple_backup_excludeItem = 'com.apple.backupd'"])
            .output()
            .context("Failed to run mdfind")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let path = std::path::PathBuf::from(line.trim());
                if !path.exists() {
                    continue;
                }
                let display_path = contract_tilde(&path.to_string_lossy());
                match excluder::remove_exclusion(&path) {
                    Ok(()) => {
                        println!("  [removed] {}", display_path);
                        removed_count += 1;
                    }
                    Err(e) => {
                        eprintln!("  [error] {}: {}", display_path, e);
                        error_count += 1;
                    }
                }
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Warning: mdfind failed: {}", stderr.trim());
        }
    } else {
        // Only remove exclusions tmignore would manage: scanned patterns + exclude_paths
        let config = config::load_config()?;
        let active_patterns =
            patterns::resolve_patterns(&config.disable_patterns, &config.custom_patterns);
        let matches = scanner::scan_optimized(&config, &active_patterns);

        for m in &matches {
            match excluder::is_excluded(&m.path) {
                Ok(true) => {
                    let display_path = contract_tilde(&m.path.to_string_lossy());
                    match excluder::remove_exclusion(&m.path) {
                        Ok(()) => {
                            println!("  [removed] {}", display_path);
                            removed_count += 1;
                        }
                        Err(e) => {
                            eprintln!("  [error] {}: {}", display_path, e);
                            error_count += 1;
                        }
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    eprintln!(
                        "  [error] checking {}: {}",
                        contract_tilde(&m.path.to_string_lossy()),
                        e
                    );
                    error_count += 1;
                }
            }
        }

        for path_str in config.resolved_exclude_paths() {
            let path = expand_tilde(&path_str);
            if path.exists() {
                if let Ok(true) = excluder::is_excluded(&path) {
                    let display_path = contract_tilde(&path.to_string_lossy());
                    match excluder::remove_exclusion(&path) {
                        Ok(()) => {
                            println!("  [removed] {}", display_path);
                            removed_count += 1;
                        }
                        Err(e) => {
                            eprintln!("  [error] {}: {}", display_path, e);
                            error_count += 1;
                        }
                    }
                }
            }
        }
    }

    // Clear state file
    let state_path = std::path::PathBuf::from(std::env::var("HOME").expect("HOME not set"))
        .join(".local/state/tmignore/state.json");
    if state_path.exists() {
        std::fs::remove_file(&state_path).ok();
    }

    println!();
    println!("  {} exclusions removed, {} errors", removed_count, error_count);

    Ok(())
}

/// Simple ISO 8601 timestamp without pulling in chrono.
fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok();

    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}
