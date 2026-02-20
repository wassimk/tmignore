# tmignore

Exclude developer dependency directories and arbitrary paths from macOS backups.

tmignore scans your filesystem for dependency directories (*node_modules*, *target*, *vendor*, *.venv*, etc.) and marks them as excluded from Time Machine using sticky exclusions (`tmutil addexclusion`). It also excludes common developer toolchain paths (Homebrew, Cargo, npm caches, Xcode DerivedData, etc.) by default.

## Install

```
brew install wassimk/tap/tmignore
```

## Usage

```
tmignore run [--dry-run] [--verbose]   # Scan and exclude
tmignore list                          # Show excluded paths from last run
tmignore add <path>                    # Add path to config + exclude immediately
tmignore remove <path>                 # Remove path from config + un-exclude
tmignore status                        # Service status and last run stats
tmignore init                          # Generate default config file
tmignore install [--force]             # Install LaunchAgent (runs every 24h)
tmignore uninstall                     # Remove LaunchAgent
tmignore reset [--all]                 # Remove backup exclusions set by tmignore
```

### Quick start

```
tmignore init                 # Create ~/.config/tmignore/config.toml
tmignore run --dry-run        # Preview what would be excluded
tmignore run                  # Exclude everything
tmignore install              # Set up background service (every 24h)
```

## Config

*~/.config/tmignore/config.toml*

tmignore ships with sensible defaults built into the binary. The config file is optional and only needed to customize behavior. Run `tmignore init` to generate one.

```toml
# Directories to scan for dependency patterns (default: home dir)
scan_roots = ["~"]

# Add extra paths to exclude from backups (on top of built-ins).
extra_exclude_paths = [
    # "~/Movies",
    # "~/Downloads",
]

# Stop excluding a built-in path (it will be backed up normally).
# disable_exclude_paths = ["~/.cargo"]

# Disable a built-in dependency pattern by name.
# disable_patterns = ["bundler"]

# Add custom dependency patterns.
# [[custom_patterns]]
# name = "my-build"
# directory = "dist"
# sentinel = "turbo.json"
```

This file is designed to be synced across machines via dotfiles, iCloud, or similar. On a new machine: `brew install wassimk/tap/tmignore && tmignore run` applies everything.

### Built-in exclude paths

These paths are excluded from backups and skipped during scanning by default. No config needed.

- **Version managers:** *~/.rbenv*, *~/.pyenv*, *~/.nvm*, *~/.asdf*, *~/.local/share/mise*
- **Language toolchains:** *~/.rustup*, *~/.cargo*, *~/.gradle*, *~/.m2*, *~/.npm*, *~/.pnpm-store*, *~/.cocoapods*, *~/.nuget*, *~/go/pkg*, *~/.gem*, *~/.hex*, *~/.cpan*, *~/.bun*, *~/.deno*, *~/.yarn*, *~/.npm-global*, *~/.cache/node*
- **Homebrew:** */opt/homebrew*
- **Nix/Devbox:** */nix*, *~/.cache/nix*, *~/.local/share/devbox*
- **Docker/Colima:** *~/Library/Containers/com.docker.docker*, *~/.colima*, *~/.lima*
- **Xcode:** *~/Library/Developer/Xcode/DerivedData*, iOS/watchOS/tvOS DeviceSupport, *~/Library/Developer/CoreSimulator/Devices*

Use `disable_exclude_paths` to stop excluding any of these. Use `extra_exclude_paths` to add your own.

## Built-in patterns

tmignore recognizes 40 dependency directory patterns. Each pattern matches a directory name and verifies a sentinel file exists in the parent directory.

| Pattern | Directory | Sentinel |
|---|---|---|
| node | node_modules | package.json |
| next | .next | package.json |
| nuxt | .nuxt | package.json |
| svelte-kit | .svelte-kit | package.json |
| angular | .angular | package.json |
| parcel | .parcel-cache | package.json |
| turbo | .turbo | package.json |
| bower | bower_components | bower.json |
| yarn | .yarn | .yarnrc.yml |
| composer | vendor | composer.json |
| bundler | vendor | Gemfile |
| cargo | target | Cargo.toml |
| go | vendor | go.mod |
| maven | target | pom.xml |
| gradle | .gradle | build.gradle |
| gradle-kts | .gradle | build.gradle.kts |
| sbt | target | build.sbt |
| swift | .build | Package.swift |
| cocoapods | Pods | Podfile |
| carthage | Carthage | Cartfile |
| flutter | .dart_tool | pubspec.yaml |
| pub | .packages | pubspec.yaml |
| python-venv | .venv | pyproject.toml |
| python-tox | .tox | tox.ini |
| python-nox | .nox | noxfile.py |
| elixir-deps | deps | mix.exs |
| elixir-build | _build | mix.exs |
| haskell | .stack-work | stack.yaml |
| vagrant | .vagrant | Vagrantfile |
| terraform | .terraform | .terraform.lock.hcl |
| terragrunt | .terragrunt-cache | terragrunt.hcl |
| cdk | cdk.out | cdk.json |
| dotnet-bin | bin | *.csproj |
| dotnet-obj | obj | *.csproj |
| zig | zig-cache | build.zig |
| ocaml | _build | dune-project |
| godot | .godot | project.godot |
| clojure | .cpcache | deps.edn |
| renv | renv | renv.lock |
| devbox | .devbox | devbox.json |

Disable any built-in pattern by adding its name to `disable_patterns` in the config. Add new patterns with `[[custom_patterns]]`.

## LaunchAgent service

`tmignore install` creates a LaunchAgent at *~/Library/LaunchAgents/com.wassimk.tmignore.plist* that runs `tmignore run` every 24 hours. Logs are written to *~/Library/Logs/tmignore/*.

The service runs in user context (not root), so `$HOME` resolves correctly and no elevated permissions are needed.

## Backup tool compatibility

The macOS exclusion metadata set by tmignore is honored by multiple backup tools:

- **Time Machine**. Native support. This is the primary target.
- **Carbon Copy Cloner**. Enable "Respect macOS exclusions" in the task filter's Source Options. CCC also supports a Global Preflight Script (Advanced > Global Preflight Script) where you can run `tmignore run` to ensure exclusions are current before every backup.
- **Backblaze**. Check "Also exclude Apple-specified exclusions" in Settings > Exclusions.
- Other backup tools that read macOS extended attributes will likely honor these exclusions too.

## How it works

tmignore uses `tmutil addexclusion` (without the `-p` flag) which writes a sticky extended attribute (`com.apple.metadata:com_apple_backup_excludeItem`) directly onto the directory. This exclusion follows the item if renamed or moved, and does not require root privileges.

## Attribution

tmignore is inspired by [asimov](https://github.com/stevegrunwell/asimov) by Steve Grunwell.
