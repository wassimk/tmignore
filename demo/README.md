# Demo GIF

`tmignore.gif` is generated from `tmignore.tape` with [VHS](https://github.com/charmbracelet/vhs). Regenerate it whenever the output or the README demo should change.

## Requirements

- [VHS](https://github.com/charmbracelet/vhs#installation) (`brew install vhs`)
- Rust, to build tmignore from this checkout
- bash
- [JetBrainsMono Nerd Font](https://www.nerdfonts.com/font-downloads) (optional, the tape tries MonoLisaCode first and VHS falls back to a default font)

## Regenerate

From the repo root:

```shell
vhs demo/tmignore.tape
```

This overwrites `demo/tmignore.gif`. Commit it along with any tape changes.

## How it works

- `setup.sh` builds a throwaway home directory in `/tmp/tmignore-demo` with a few fake projects, each with a dependency directory (`node_modules`, `vendor`, `target`, `.venv`).
- It builds tmignore from this checkout and writes `env.sh`, which points `HOME` at the sandbox and puts the shims in `demo/shims` first on `PATH`.
- The shims stand in for `tmutil`, `launchctl`, and `du`. Recording never scans your real home directory, never changes Time Machine exclusions, and never loads or unloads the real LaunchAgent, even if tmignore is installed on the machine.
- The `du` shim reports the size in each directory's `.demo-size` file, so the demo shows realistic sizes without writing gigabytes to disk.
- Steps wait on screen text (`Wait+Screen`), so if the output changes the render fails instead of recording a broken demo.
