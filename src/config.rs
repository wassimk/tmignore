use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomPattern {
    pub name: String,
    pub directory: String,
    pub sentinel: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_scan_roots")]
    pub scan_roots: Vec<String>,

    /// Additional paths to exclude from backups (on top of built-ins).
    #[serde(default)]
    pub extra_exclude_paths: Vec<String>,

    /// Built-in exclude paths to disable (these will be backed up normally).
    #[serde(default)]
    pub disable_exclude_paths: Vec<String>,

    #[serde(default)]
    pub disable_patterns: Vec<String>,

    #[serde(default)]
    pub custom_patterns: Vec<CustomPattern>,
}

fn default_scan_roots() -> Vec<String> {
    vec!["~".to_string()]
}

/// System directories the scanner should never walk into.
/// These are not excluded from backups, just skipped for scanning.
const SYSTEM_SKIP_PATHS: &[&str] = &[
    "~/.Trash",
    "~/Library",
    "/System",
    "/Library",
];

/// Built-in paths to exclude from backups.
/// These are large, fully regenerable directories.
/// They are also automatically skipped during scanning.
pub fn builtin_exclude_paths() -> Vec<&'static str> {
    vec![
        // Version managers
        "~/.rbenv",
        "~/.pyenv",
        "~/.nvm",
        "~/.asdf",
        "~/.local/share/mise",
        // Language toolchain caches
        "~/.rustup",
        "~/.cargo",
        "~/.gradle",
        "~/.m2",
        "~/.npm",
        "~/.pnpm-store",
        "~/.cocoapods",
        "~/.nuget",
        "~/go/pkg",
        "~/.gem",
        "~/.hex",
        "~/.cpan",
        "~/.bun",
        "~/.deno",
        "~/.yarn",
        "~/.npm-global",
        "~/.cache/node",
        // Homebrew
        "/opt/homebrew",
        // Nix / Devbox
        "/nix",
        "~/.cache/nix",
        "~/.local/share/devbox",
        // Docker / Colima
        "~/Library/Containers/com.docker.docker",
        "~/.colima",
        "~/.lima",
        // Xcode
        "~/Library/Developer/Xcode/DerivedData",
        "~/Library/Developer/Xcode/iOS DeviceSupport",
        "~/Library/Developer/Xcode/watchOS DeviceSupport",
        "~/Library/Developer/Xcode/tvOS DeviceSupport",
        "~/Library/Developer/CoreSimulator/Devices",
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_roots: default_scan_roots(),
            extra_exclude_paths: Vec::new(),
            disable_exclude_paths: Vec::new(),
            disable_patterns: Vec::new(),
            custom_patterns: Vec::new(),
        }
    }
}

impl Config {
    /// Resolve effective exclude paths: built-ins minus disabled, plus extras.
    pub fn resolved_exclude_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = builtin_exclude_paths()
            .into_iter()
            .filter(|p| !self.disable_exclude_paths.iter().any(|d| d == p))
            .map(|p| p.to_string())
            .collect();

        for extra in &self.extra_exclude_paths {
            if !paths.contains(extra) {
                paths.push(extra.clone());
            }
        }

        paths
    }

    /// Resolve paths the scanner should skip: system paths + all resolved exclude paths.
    pub fn resolved_skip_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = SYSTEM_SKIP_PATHS
            .iter()
            .map(|p| p.to_string())
            .collect();

        for p in self.resolved_exclude_paths() {
            if !paths.contains(&p) {
                paths.push(p);
            }
        }

        paths
    }

    pub fn default_toml() -> &'static str {
        r#"# Directories to scan for dependency patterns (default: home dir)
scan_roots = ["~"]

# tmignore excludes these paths from backups by default:
# version managers (~/.rbenv, ~/.pyenv, ~/.nvm, ~/.asdf, ~/.local/share/mise),
# language toolchain caches (~/.cargo, ~/.rustup, ~/.gradle, ~/.m2, ~/.npm, etc.),
# Homebrew (/opt/homebrew), Nix/Devbox (/nix), Docker, and
# Xcode (DerivedData, DeviceSupport, CoreSimulator).
#
# Run `tmignore run --verbose` to see the full list.

# Add extra paths to exclude from backups (on top of built-ins).
# Supports ~ expansion.
extra_exclude_paths = [
    # Virtual machines
    # "~/Parallels",
    # "~/Virtual Machines.localized",
    # "~/.vagrant.d/boxes",

    # Android
    # "~/Library/Android/sdk",
    # "~/.android/avd",

    # Large media (user preference)
    # "~/Movies",
    # "~/Downloads",
]

# Stop excluding a built-in path (it will be backed up normally).
# disable_exclude_paths = ["~/.cargo"]

# tmignore scans for dependency directories (node_modules, target, vendor, etc.)
# by matching a directory name + a sentinel file in its parent (e.g. package.json).
# 40 patterns are built-in. You can disable any by name or add your own.
#
# disable_patterns = ["bundler"]
#
# [[custom_patterns]]
# name = "my-build"
# directory = "dist"
# sentinel = "turbo.json"
"#
    }
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/tmignore")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(rest)
    } else if path == "~" {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home)
    } else {
        PathBuf::from(path)
    }
}

pub fn contract_tilde(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if let Some(rest) = path.strip_prefix(&home) {
            if rest.is_empty() {
                return "~".to_string();
            }
            if rest.starts_with('/') {
                return format!("~{rest}");
            }
        }
    }
    path.to_string()
}

pub fn load_config() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents =
        std::fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;

    let config: Config =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", path.display()))?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path();
    std::fs::create_dir_all(config_dir()).context("Failed to create config directory")?;
    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;
    std::fs::write(&path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.scan_roots, vec!["~"]);
        assert!(config.extra_exclude_paths.is_empty());
        assert!(config.disable_exclude_paths.is_empty());
        assert!(config.disable_patterns.is_empty());
        assert!(config.custom_patterns.is_empty());
    }

    #[test]
    fn test_resolved_exclude_paths_includes_builtins() {
        let config = Config::default();
        let resolved = config.resolved_exclude_paths();
        assert!(resolved.contains(&"~/.rbenv".to_string()));
        assert!(resolved.contains(&"~/Library/Developer/Xcode/DerivedData".to_string()));
    }

    #[test]
    fn test_resolved_exclude_paths_disable() {
        let config = Config {
            disable_exclude_paths: vec!["~/.cargo".to_string()],
            ..Config::default()
        };
        let resolved = config.resolved_exclude_paths();
        assert!(!resolved.contains(&"~/.cargo".to_string()));
        assert!(resolved.contains(&"~/.rbenv".to_string()));
    }

    #[test]
    fn test_resolved_exclude_paths_extra() {
        let config = Config {
            extra_exclude_paths: vec!["~/Movies".to_string()],
            ..Config::default()
        };
        let resolved = config.resolved_exclude_paths();
        assert!(resolved.contains(&"~/Movies".to_string()));
        assert!(resolved.contains(&"~/.rbenv".to_string()));
    }

    #[test]
    fn test_resolved_skip_paths_includes_system_and_excludes() {
        let config = Config::default();
        let resolved = config.resolved_skip_paths();
        // System paths
        assert!(resolved.contains(&"~/.Trash".to_string()));
        assert!(resolved.contains(&"~/Library".to_string()));
        // Exclude paths also get skipped
        assert!(resolved.contains(&"~/.rbenv".to_string()));
        assert!(resolved.contains(&"~/.cargo".to_string()));
    }

    #[test]
    fn test_disabled_exclude_not_in_skip() {
        let config = Config {
            disable_exclude_paths: vec!["~/.cargo".to_string()],
            ..Config::default()
        };
        let resolved = config.resolved_skip_paths();
        // Disabled exclude path should NOT be skipped (scanner walks into it)
        assert!(!resolved.contains(&"~/.cargo".to_string()));
        // System paths still present
        assert!(resolved.contains(&"~/Library".to_string()));
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
scan_roots = ["~", "/Volumes/Code"]
extra_exclude_paths = ["~/Movies"]
disable_patterns = ["node"]
disable_exclude_paths = ["~/.cargo"]

[[custom_patterns]]
name = "my-build"
directory = "dist"
sentinel = "turbo.json"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.scan_roots, vec!["~", "/Volumes/Code"]);
        assert_eq!(config.extra_exclude_paths, vec!["~/Movies"]);
        assert_eq!(config.disable_patterns, vec!["node"]);
        assert_eq!(config.disable_exclude_paths, vec!["~/.cargo"]);
        assert_eq!(config.custom_patterns.len(), 1);
        assert_eq!(config.custom_patterns[0].name, "my-build");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/Documents");
        assert!(!expanded.to_string_lossy().contains('~'));
        assert!(expanded.to_string_lossy().ends_with("/Documents"));

        let expanded_home = expand_tilde("~");
        assert!(!expanded_home.to_string_lossy().contains('~'));

        let absolute = expand_tilde("/usr/local");
        assert_eq!(absolute, PathBuf::from("/usr/local"));
    }

    #[test]
    fn test_contract_tilde() {
        let home = std::env::var("HOME").unwrap();
        let contracted = contract_tilde(&format!("{home}/Documents"));
        assert_eq!(contracted, "~/Documents");

        let contracted_home = contract_tilde(&home);
        assert_eq!(contracted_home, "~");

        let absolute = contract_tilde("/usr/local");
        assert_eq!(absolute, "/usr/local");
    }

    #[test]
    fn test_default_toml_parses() {
        let _config: Config = toml::from_str(Config::default_toml()).unwrap();
    }

    #[test]
    fn test_builtin_exclude_count() {
        assert!(builtin_exclude_paths().len() >= 20);
    }
}
