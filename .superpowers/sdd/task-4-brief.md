### Task 4: Windows Platform Adapter

Implement Windows discovery, icon lookup, running detection, shortcut resolution, and direct launch as a cohesive Windows adapter. This branch starts from Task 2 while Task 3 runs in parallel, so keep Windows-specific logic behind a narrow public API and do not invent a competing shared trait; Task 5 will connect both adapters after merge. Clearly document proposed shared inputs/results. Work only in this worktree and Rust Windows/adapter modules/tests. Linux code, orchestration, settings UI, and logs are out of scope.

Required behavior:

- Windows-only production code must compile behind `cfg(windows)`; fixture/parsing/mapping logic must remain testable on Fedora.
- Read HKCU/HKLM App Paths registrations first, then uninstall/installed-app records and user/common Start Menu shortcuts. No full-disk scan.
- Resolve `.lnk` targets via Windows Shell/COM APIs on Windows, with injectable fixture resolver elsewhere. Reject non-file/UWP-only targets in v1.
- Detect Brave and Antigravity using registry records, known install locations, then expose unresolved discovery so later settings UI can use executable picker. Do not hard-fail nonstandard installs.
- Normalize/dedupe paths and names without trusting display strings as executable commands.
- Extract application icons through Windows Shell APIs into a safe local/data representation; explicit generic fallback on failure. Production calls gated to Windows.
- Running detection compares canonical executable paths first; known Brave/Antigravity aliases second; permission errors/ambiguous names return `Unknown`.
- Launch only resolved absolute executable paths via `std::process::Command` argument arrays. Never use `cmd.exe`, PowerShell, `ShellExecute` command strings, or a Tauri shell plugin.
- Add no real registry mutation, app launch, or external network behavior.

Tests first using fixtures: App Paths precedence, per-user/machine records, uninstall entries, Start Menu `.lnk` resolution, UWP exclusion, dedupe, known-location fallback, icon fallback, exact/alias/unknown running state, argument preservation, cfg compilation. Run full checks available on Fedora; explicitly record that Windows live compile/runtime verification remains Task 9. Self-review, report `.superpowers/sdd/task-4-report.md`, commit, return SHA.
