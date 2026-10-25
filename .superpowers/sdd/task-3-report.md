# Task 3: Linux Platform Adapter Report

## Delivered

- Added a platform-neutral adapter boundary with injectable process snapshots and
  process spawning.
- Implemented fixture-driven XDG desktop-entry discovery with user precedence,
  tombstone masking, localized names, visibility checks, and `TryExec`
  validation.
- Added safe desktop `Exec` parsing: quoting and freedesktop field-code handling
  produce an executable plus argument array without shell evaluation. Shells and
  `env` wrappers are rejected.
- Added direct executable resolution, constrained DBus-only `gtk-launch`
  fallback, icon lookup with an explicit fallback, and direct `Command` spawning.
- Added conservative running detection using canonical executable paths, known
  Brave/Antigravity aliases, wrapper uncertainty, and ambiguity handling.

## Verification

- `cargo fmt --check`
- `cargo test` — 38 passed, 0 failed, 1 ignored host-smoke test
- `cargo test --test linux_adapter manual_host_discovery_finds_brave_and_antigravity -- --ignored --exact` — passed
- `pnpm test -- --run` — 2 passed
- `pnpm check` — 0 errors, 0 warnings
- `pnpm tauri build --debug --no-bundle` — passed

The host smoke test is intentionally ignored in the default suite; it remains
read-only and confirms the installed Antigravity and Brave desktop entries can
be discovered and resolved on this Fedora workstation.
