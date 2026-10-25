### Task 6: Main Svelte Interface

Implement the Workstart main window UI against a fixed frontend IPC contract, independently of unfinished backend internals. Work only in this task worktree and frontend files under `src/` plus frontend tests/dependencies.

Contract:

- `get_workflow` returns `{ schemaVersion: number, apps: AppTarget[] }` where an app has `id`, `name`, `launchRef`, `arguments`, `processMatchers`, `enabled`, optional `iconDataUrl`, and optional `validationError`.
- `launch_targets` accepts `{ targetIds: string[] }` and returns one result per ID: `{ targetId, outcome: 'started' | 'skippedAlreadyRunning' | 'failed', errorCategory?: string }`.
- Settings window opens using Tauri WebviewWindow label `settings`; app exit uses the Tauri process exit API only after full success and a 1-second confirmation delay.
- Put IPC behind an injectable service module so Vitest uses fakes and no Tauri runtime.

Required UI:

- Exact 440x320-friendly layout: standard title bar outside web content, header title/settings icon, compact scrollable target list, large single Start Work button.
- Statuses Ready, Launching, Already open, Started, Failed. Concurrent operation represented per item.
- Disabled if no enabled valid target. Partial failure changes button to Try Again and retries failed IDs only. Skipped counts as success. Full success shows confirmation then closes after one second.
- System light/dark, cool neutral palette, one cobalt accent, restrained simulated frosted surfaces inside opaque window, 12px radius, 120-180ms transitions, reduced-motion, keyboard focus, AA contrast, long-name/empty/error states.
- Use inline/local SVG icons or a small icon dependency; no emoji and no remote assets.
- Svelte Testing Library tests first for idle/loading/full success/skip/partial failure/retry/disabled/one-second close/settings activation and accessibility basics.

Do not implement settings editing in this task. Read approved plan and design spec. Run frontend tests/check/build plus existing Rust tests. Self-review, write `.superpowers/sdd/task-6-report.md`, commit, and return SHA.
