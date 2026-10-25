# Workstart context snapshot

## Goal

Finish Workstart v0.1.0: Tauri 2 launcher for Fedora 44 and Windows 10/11, preloaded with Antigravity IDE and Brave. Merge reviewed task branches, verify locally, then publish to the user's GitHub repository after authentication.

## Current commits

- `eb9889f` bootstrap/test harness
- `3100391` Rust domain/config persistence
- `b8ecd78` main Svelte interface
- `147ab63` packaging/icons/release docs

## In progress

- Core IPC view DTO fix: `feature/workstart-core` dirty
- Linux adapter: `feature/workstart-linux` dirty
- Windows adapter: `feature/workstart-windows` dirty

## Constraints

- No shell plugin or shell command strings.
- Stored target IDs only; local config/logs; no telemetry.
- Review every task before integration.
- Do not claim complete before fresh full verification.
- Windows live verification, Fedora GNOME, RPM generation, code signing, and GitHub publishing remain environment-dependent.

## GitHub blocker

Remote absent. `gh auth status` reports invalid token for `chx-n`; run `gh auth login -h github.com` in user's terminal, then configure remote `https://github.com/chx-n/workstart.git` after confirming repository name.
