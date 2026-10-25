# Task 4: Windows Platform Adapter Report

## Delivered

- Added a Windows-only native adapter behind `cfg(windows)`. It reads HKCU/HKLM
  App Paths and uninstall records, enumerates user/common Start Menu shortcuts,
  uses Shell/COM to resolve `.lnk` files, and checks known Brave and Antigravity
  locations without scanning disks.
- Kept discovery parsing and mapping host-independent through injected shortcut
  and canonical-file callbacks. Resolved applications contain only validated,
  absolute `.exe` paths; UWP/shell targets and command tails are rejected.
- Added explicit unresolved `Brave`/`Antigravity` candidates for the future
  settings executable picker. The Task 5 handoff is `Application` for a
  canonical resolved target and `UnresolvedApplication` for a known missing
  default; no shared platform trait was introduced.
- Added checked direct-launch plans that preserve argument-array boundaries.
  The Windows launcher uses `std::process::Command` only; there are no shell
  strings or Tauri shell-plugin calls.
- Added path-first running detection with known aliases only when no process
  path is readable, plus validated RGBA icon extraction with an explicit generic
  fallback.

## Verification

- `cargo fmt -- --check`
- `cargo test` — 27 tests passed, including 9 Windows-adapter fixture tests.
- Attempted `cargo check --target x86_64-pc-windows-gnu`. The Rust target and
  adapter dependencies are available, but Tauri's Windows resource build cannot
  run on this Fedora host because `x86_64-w64-mingw32-windres` is not installed.
  A temporary ICO was generated only to get past the repository's missing
  Windows icon, then removed; no generated icon is included in this task.

Windows live compilation and runtime behavior remain Task 9 validation.
