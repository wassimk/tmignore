# README demo for tmignore
#
# Render from the repo root:
#   vhs demo/tmignore.tape
#
# demo/setup.sh builds a throwaway home directory in /tmp/tmignore-demo and a fresh tmignore
# binary. Shims for tmutil, launchctl, and du keep the demo away from real backups and services.

Output demo/tmignore.gif

Require bash
Require cargo

Set Shell "bash"
Set FontFamily "MonoLisaCode, JetBrainsMono Nerd Font"
Set FontSize 16
Set Width 1000
Set Height 540
Set Padding 20
Set Theme "OneHalfDark"
Set TypingSpeed 60ms
Set CursorBlink false

# Setup: sandbox home, demo environment, clean prompt
Hide
Type "bash demo/setup.sh && source /tmp/tmignore-demo/env.sh"
Enter
Wait@120s />\s*$/
Type "PS1='\[\e[36m\]~\[\e[0m\] ❯ ' && clear"
Enter
Sleep 500ms
Show

Sleep 800ms

# One run excludes every dependency directory it finds
Type "tmignore run"
Sleep 500ms
Enter
Wait+Screen@20s /newly excluded/
Sleep 3.5s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# Install the background service, then confirm it is loaded
Type "tmignore install"
Sleep 500ms
Enter
Wait+Screen /every 24 hours/
Sleep 2.5s

Type "tmignore status"
Sleep 500ms
Enter
Wait+Screen /Logs:   ~/

# Hold on the result before looping
Sleep 4s
