use crate::config::CustomPattern;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub directory: String,
    pub sentinel: String,
}

impl Pattern {
    fn new(name: &str, directory: &str, sentinel: &str) -> Self {
        Self {
            name: name.to_string(),
            directory: directory.to_string(),
            sentinel: sentinel.to_string(),
        }
    }
}

impl From<&CustomPattern> for Pattern {
    fn from(cp: &CustomPattern) -> Self {
        Self {
            name: cp.name.clone(),
            directory: cp.directory.clone(),
            sentinel: cp.sentinel.clone(),
        }
    }
}

pub fn builtin_patterns() -> Vec<Pattern> {
    vec![
        Pattern::new("node", "node_modules", "package.json"),
        Pattern::new("next", ".next", "package.json"),
        Pattern::new("nuxt", ".nuxt", "package.json"),
        Pattern::new("svelte-kit", ".svelte-kit", "package.json"),
        Pattern::new("angular", ".angular", "package.json"),
        Pattern::new("parcel", ".parcel-cache", "package.json"),
        Pattern::new("turbo", ".turbo", "package.json"),
        Pattern::new("bower", "bower_components", "bower.json"),
        Pattern::new("yarn", ".yarn", ".yarnrc.yml"),
        Pattern::new("composer", "vendor", "composer.json"),
        Pattern::new("bundler", "vendor", "Gemfile"),
        Pattern::new("cargo", "target", "Cargo.toml"),
        Pattern::new("go", "vendor", "go.mod"),
        Pattern::new("maven", "target", "pom.xml"),
        Pattern::new("gradle", ".gradle", "build.gradle"),
        Pattern::new("gradle-kts", ".gradle", "build.gradle.kts"),
        Pattern::new("sbt", "target", "build.sbt"),
        Pattern::new("swift", ".build", "Package.swift"),
        Pattern::new("cocoapods", "Pods", "Podfile"),
        Pattern::new("carthage", "Carthage", "Cartfile"),
        Pattern::new("flutter", ".dart_tool", "pubspec.yaml"),
        Pattern::new("pub", ".packages", "pubspec.yaml"),
        Pattern::new("python-venv", ".venv", "pyproject.toml"),
        Pattern::new("python-tox", ".tox", "tox.ini"),
        Pattern::new("python-nox", ".nox", "noxfile.py"),
        Pattern::new("elixir-deps", "deps", "mix.exs"),
        Pattern::new("elixir-build", "_build", "mix.exs"),
        Pattern::new("haskell", ".stack-work", "stack.yaml"),
        Pattern::new("vagrant", ".vagrant", "Vagrantfile"),
        Pattern::new("terraform", ".terraform", ".terraform.lock.hcl"),
        Pattern::new("terragrunt", ".terragrunt-cache", "terragrunt.hcl"),
        Pattern::new("cdk", "cdk.out", "cdk.json"),
        Pattern::new("dotnet-bin", "bin", "*.csproj"),
        Pattern::new("dotnet-obj", "obj", "*.csproj"),
        Pattern::new("zig", "zig-cache", "build.zig"),
        Pattern::new("ocaml", "_build", "dune-project"),
        Pattern::new("godot", ".godot", "project.godot"),
        Pattern::new("clojure", ".cpcache", "deps.edn"),
        Pattern::new("renv", "renv", "renv.lock"),
        Pattern::new("devbox", ".devbox", "devbox.json"),
    ]
}

/// Resolve active patterns: built-ins minus disabled, plus custom patterns.
pub fn resolve_patterns(disable: &[String], custom: &[CustomPattern]) -> Vec<Pattern> {
    let mut patterns: Vec<Pattern> = builtin_patterns()
        .into_iter()
        .filter(|p| !disable.iter().any(|d| d == &p.name))
        .collect();

    for cp in custom {
        patterns.push(Pattern::from(cp));
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_count() {
        let patterns = builtin_patterns();
        assert!(patterns.len() >= 35, "Expected at least 35 patterns, got {}", patterns.len());
    }

    #[test]
    fn test_resolve_patterns_disable() {
        let patterns = resolve_patterns(&["node".to_string(), "cargo".to_string()], &[]);
        assert!(!patterns.iter().any(|p| p.name == "node"));
        assert!(!patterns.iter().any(|p| p.name == "cargo"));
        assert!(patterns.iter().any(|p| p.name == "next"));
    }

    #[test]
    fn test_resolve_patterns_custom() {
        let custom = vec![CustomPattern {
            name: "my-build".to_string(),
            directory: "dist".to_string(),
            sentinel: "turbo.json".to_string(),
        }];
        let patterns = resolve_patterns(&[], &custom);
        assert!(patterns.iter().any(|p| p.name == "my-build"));
    }

    #[test]
    fn test_all_patterns_have_fields() {
        for p in builtin_patterns() {
            assert!(!p.name.is_empty(), "Pattern has empty name");
            assert!(!p.directory.is_empty(), "Pattern {} has empty directory", p.name);
            assert!(!p.sentinel.is_empty(), "Pattern {} has empty sentinel", p.name);
        }
    }
}
