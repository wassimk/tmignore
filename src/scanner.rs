use crate::config::{expand_tilde, Config};
use crate::patterns::Pattern;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Check if a sentinel file exists in the given parent directory.
/// Supports glob patterns (e.g., "*.csproj").
fn sentinel_exists(parent: &Path, sentinel: &str) -> bool {
    if sentinel.contains('*') || sentinel.contains('?') || sentinel.contains('[') {
        // Glob pattern
        let pattern = parent.join(sentinel).to_string_lossy().to_string();
        if let Ok(entries) = glob::glob(&pattern) {
            return entries.filter_map(|e| e.ok()).next().is_some();
        }
        false
    } else {
        // Exact file name
        parent.join(sentinel).exists()
    }
}

/// Build the set of directories to skip during scanning.
fn build_skip_set(config: &Config) -> HashSet<PathBuf> {
    config.resolved_skip_paths().iter().map(|p| expand_tilde(p)).collect()
}

/// Build a lookup of directory name -> list of patterns for fast matching.
fn build_directory_index(patterns: &[Pattern]) -> std::collections::HashMap<String, Vec<&Pattern>> {
    let mut index: std::collections::HashMap<String, Vec<&Pattern>> = std::collections::HashMap::new();
    for p in patterns {
        index.entry(p.directory.clone()).or_default().push(p);
    }
    index
}

/// Result of a scan: path to exclude, matched pattern name, and whether it came from a pattern or exclude_paths.
#[derive(Debug)]
pub struct ScanMatch {
    pub path: PathBuf,
    pub pattern_name: String,
}

/// Scan all configured roots for dependency directories matching the given patterns.
/// Skips descending into matched dependency directories for performance.
pub fn scan_optimized(config: &Config, patterns: &[Pattern]) -> Vec<ScanMatch> {
    let skip_set = build_skip_set(config);
    let dir_index = build_directory_index(patterns);
    let mut matches = Vec::new();
    let mut excluded_dirs: HashSet<PathBuf> = HashSet::new();

    for root_str in &config.scan_roots {
        let root = expand_tilde(root_str);

        if !root.exists() {
            eprintln!("Warning: scan root does not exist: {}", root.display());
            continue;
        }

        let mut walker = WalkDir::new(&root).follow_links(false).into_iter();

        loop {
            let entry = match walker.next() {
                Some(Ok(e)) => e,
                Some(Err(err)) => {
                    if let Some(path) = err.path() {
                        eprintln!("Warning: cannot access {}: {}", path.display(), err);
                    }
                    continue;
                }
                None => break,
            };

            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path().to_path_buf();

            // Skip paths in skip set
            if skip_set.contains(&path) {
                walker.skip_current_dir();
                continue;
            }

            // Skip already-matched dependency directories (no point descending into node_modules)
            if excluded_dirs.contains(&path) {
                walker.skip_current_dir();
                continue;
            }

            let dir_name = match entry.file_name().to_str() {
                Some(name) => name.to_string(),
                None => continue,
            };

            if let Some(candidates) = dir_index.get(&dir_name) {
                if let Some(parent) = path.parent() {
                    for pattern in candidates {
                        if sentinel_exists(parent, &pattern.sentinel) {
                            excluded_dirs.insert(path.clone());
                            matches.push(ScanMatch {
                                path: path.clone(),
                                pattern_name: pattern.name.clone(),
                            });
                            walker.skip_current_dir();
                            break;
                        }
                    }
                }
            }
        }
    }

    // Add resolved exclude_paths (built-ins + extras - disabled)
    for path_str in config.resolved_exclude_paths() {
        let path = expand_tilde(&path_str);
        if path.exists() {
            matches.push(ScanMatch {
                path,
                pattern_name: "exclude_path".to_string(),
            });
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sentinel_exists_exact() {
        let dir = std::env::temp_dir().join("tmignore_test_sentinel");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("package.json"), "{}").unwrap();

        assert!(sentinel_exists(&dir, "package.json"));
        assert!(!sentinel_exists(&dir, "Cargo.toml"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sentinel_exists_glob() {
        let dir = std::env::temp_dir().join("tmignore_test_glob");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("MyApp.csproj"), "<Project/>").unwrap();

        assert!(sentinel_exists(&dir, "*.csproj"));
        assert!(!sentinel_exists(&dir, "*.fsproj"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_finds_node_modules() {
        let dir = std::env::temp_dir().join("tmignore_test_scan");
        let _ = fs::remove_dir_all(&dir);
        let project_dir = dir.join("myproject");
        fs::create_dir_all(project_dir.join("node_modules/.package-lock.json")).unwrap();
        fs::write(project_dir.join("package.json"), "{}").unwrap();

        // Disable all built-in exclude paths so we only see scan results
        let disable_all_excludes: Vec<String> = crate::config::builtin_exclude_paths()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let config = Config {
            scan_roots: vec![dir.to_string_lossy().to_string()],
            extra_exclude_paths: vec![],
            disable_exclude_paths: disable_all_excludes,
            disable_patterns: vec![],
            custom_patterns: vec![],
        };

        let patterns = vec![Pattern {
            name: "node".to_string(),
            directory: "node_modules".to_string(),
            sentinel: "package.json".to_string(),
        }];

        let matches = scan_optimized(&config, &patterns);
        assert!(matches.iter().any(|m| m.pattern_name == "node" && m.path.ends_with("node_modules")));

        let _ = fs::remove_dir_all(&dir);
    }
}
