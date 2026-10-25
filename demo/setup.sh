#!/usr/bin/env bash
#
# Builds a throwaway home directory in /tmp/tmignore-demo for recording the README demo
# (see demo/tmignore.tape). It holds a few fake projects with dependency directories.
#
# tmignore runs with HOME pointed at the sandbox and with shims for tmutil, launchctl, and du
# first on PATH, so the demo never scans the real home directory, never changes Time Machine
# exclusions, and never touches the real LaunchAgent.

set -euo pipefail

DEMO_DIR=/tmp/tmignore-demo
REPO_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
DEMO_HOME=$DEMO_DIR/home

rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR/bin" "$DEMO_HOME"

# project <path> <sentinel file> <dependency dir> <size shown by du>
project() {
  mkdir -p "$DEMO_HOME/projects/$1/src" "$DEMO_HOME/projects/$1/$3"
  touch "$DEMO_HOME/projects/$1/$2"
  echo "$4" >"$DEMO_HOME/projects/$1/$3/.demo-size"
}

project storefront package.json node_modules 412M
project storefront/docs package.json node_modules 96M
project billing-api Gemfile vendor 188M
project sync-cli Cargo.toml target 1.3G
project forecast pyproject.toml .venv 534M

# Built-in paths outside the sandbox that the real tool would find already excluded
printf '%s\n' /opt/homebrew /nix >"$DEMO_DIR/exclusions"

cargo build --release --quiet --manifest-path "$REPO_DIR/Cargo.toml"
cp "$REPO_DIR/target/release/tmignore" "$DEMO_DIR/bin/"

cat >"$DEMO_DIR/env.sh" <<ENV
export TMIGNORE_DEMO_DIR=$DEMO_DIR
export PATH=$DEMO_DIR/bin:$REPO_DIR/demo/shims:\$PATH
export HOME=$DEMO_HOME
cd "\$HOME"
ENV
