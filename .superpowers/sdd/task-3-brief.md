### Task 3: Linux Platform Adapter

Implement Linux discovery, icon lookup, running detection, and launch resolution behind a reusable `PlatformAdapter` boundary. Work only in this worktree and Rust adapter modules/tests. Do not implement Windows behavior, batch orchestration, settings UI, or log rotation.

Required behavior:

- Define platform-neutral adapter traits/request-result types needed by later orchestration. Keep OS calls injectable/testable.
- Discover `.desktop` entries from XDG user dir before system dirs. Honor user override/tombstone precedence.
- Parse valid `Type=Application`; reject/skip `Hidden=true`, `NoDisplay=true`, invalid entries, failed `TryExec`, and unsupported launch-only cases as appropriate.
- Use localized `Name` based on locale fallback. Parse `Exec` tokenization/quoting and freedesktop field codes correctly; remove supported field codes without shell evaluation. Never invoke `/bin/sh`, `sh -c`, or construct shell strings.
- Resolve executable directly where possible. Permit `gtk-launch <desktop-id>` only for DBus-activated entries with no usable executable and no custom arguments.
- Resolve absolute icons and common themed icon directories with scale/extension preference; return explicit fallback on failure. Do not redistribute third-party icons.
- Detect running processes from injectable snapshots, preferring canonical executable paths. Support Brave aliases and default Antigravity path/name. Return `Unknown` for wrappers, permission failures, unreadable paths, or ambiguity. Skip only belongs to later orchestration.
- Launch resolved executable via `std::process::Command` with argument array. Tests must prove values remain separate and shell metacharacters are inert arguments. Use injected spawner in unit tests; no real app launches.
- Discovery should identify installed local Antigravity and Brave on this Fedora host when an opt-in ignored/manual smoke command is run, but automated tests use fixtures.
- Add narrow discovery/icon command wiring only if stable with current domain contract; no arbitrary frontend paths/commands.

Tests first: XDG precedence, tombstones, visibility, `TryExec`, locale names, Exec quoting/field codes, icon lookup, direct/DBus resolution, exact/alias/unknown running state, argument preservation. Run full Rust/frontend/Tauri checks. Self-review, report `.superpowers/sdd/task-3-report.md`, commit, return SHA.
