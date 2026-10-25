# Task 5: Launch Orchestration and Logs

## Delivered

- Added the `launch_targets({ targetIds })` Tauri command. It loads the machine-local workflow and resolves each supplied ID against stored targets only; executable paths and argument arrays are never accepted from IPC.
- Added a narrow `TargetRuntime` orchestration boundary. Linux uses the existing XDG/Linux adapter and a conservative `/proc` snapshot; Windows uses its cfg-gated native adapter, resolving only saved executable references or stored Windows application names.
- Launches eligible targets concurrently, preserves the caller's result order, skips only a confirmed `Running` target, and launches `Unknown` targets normally. Every requested ID gets an independent result, so failed IDs can be retried directly.
- Added structured JSONL local diagnostics under the application config directory: redacted fields include only timestamp, target ID, outcome, and error category. Argument values, paths, target names, and process data are omitted.
- Added synchronized rotation with one active `launch.log` plus six history files, each capped at 1 MiB. Log failures do not change launch results.

## TDD Evidence

- RED: `cargo test --manifest-path src-tauri/Cargo.toml --test launch_orchestration` initially failed because the requested `orchestration` module did not exist.
- GREEN: four new orchestration tests now cover confirmed-running skip plus Unknown launch, independent missing/failed targets, structured argument-redacted logs, and bounded rotation.

## Verification

- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — passed: 60 tests passed, 1 host-discovery test ignored.
- `pnpm exec vitest run src/App.test.ts src/lib/workstart-service.test.ts` — passed: 11 files, 54 tests.
- `pnpm check` — passed: 0 errors, 0 warnings.
- `pnpm build` — passed.
- `git diff --check` — passed.
- `pnpm test` also discovers nested `.worktrees/workstart-settings` tests and fails one pre-existing Settings expectation there; no files in that worktree were changed.
- `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu` is blocked by the host lacking `x86_64-w64-mingw32-windres`, the existing Tauri Windows resource-build requirement. Native Windows compilation/runtime validation remains Task 9 work.

## Scope

No settings UI was implemented or changed.
