### Task 2: Domain Model and Configuration Persistence

Implement the Rust domain types and versioned machine-local workflow persistence from the approved plan. Work only in this task worktree and primarily under `src-tauri/src/domain` and `src-tauri/src/config` (plus narrow wiring/tests as needed).

Required behavior:

- `WorkflowConfig`, `AppTarget`, `LaunchRef`, `ProcessMatcher`, `RunningState`, `LaunchOutcome`, and `ErrorCategory` are serializable, strongly typed, and platform-neutral.
- Default config contains enabled Antigravity IDE and Brave targets with stable UUIDs and Fedora desktop IDs/path hints. Windows paths may remain discovery-resolved.
- Load/save versioned JSON in an injected config directory so tests never touch the real user directory.
- Atomic write through same-directory temporary file, sync, then rename. Preserve last valid config if a write fails.
- Recover from missing config with defaults. Treat corrupt/unsupported data as an explicit recoverable load result while preserving/renaming the bad file rather than silently deleting it.
- Validate schema version, non-empty trimmed names, unique IDs, target count, argument count/length, matcher count/length, and launch references. Validation must be platform-aware enough that Windows paths can be stored on Linux without requiring existence; existence checks happen before launch/discovery.
- No secrets. No shell command strings.
- Add narrow Tauri commands `get_workflow` and `save_workflow` backed by managed state. Frontend data cannot alter config location.
- Tests first for defaults, round-trip, corruption, atomic-save safety, duplicate IDs, bad refs, and argument limits.

Global constraints: read `docs/superpowers/plans/2026-10-23-workstart-v1.md`; use TDD; no shell plugin; no telemetry/network. Run Rust tests/fmt/clippy plus existing frontend checks. Self-review, write `.superpowers/sdd/task-2-report.md`, commit, and return commit SHA.
