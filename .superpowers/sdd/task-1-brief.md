### Task 1: Bootstrap and Test Harness

Create a Tauri 2 + Svelte 5 + TypeScript + Vite project using pnpm. Configure package identity, main/settings windows, restrictive capabilities/CSP, single-instance behavior, Vitest, Svelte Testing Library, Rust test support, MIT license, README, and structure documentation. Do not add shell plugin. Verify frontend tests, Rust tests, typecheck, and production frontend build. Commit the task.

## Binding Global Constraints

- Identifier: `io.github.chxnn.workstart`; version: `0.1.0`; license: MIT.
- Fedora 44 KDE/GNOME and Windows 10/11 x86-64 only.
- Never invoke a shell or accept shell command strings.
- Launch stored target IDs only; backend resolves executable configuration.
- Local machine configuration and logs only; no network telemetry.
- Main window 440x320; settings window 760x600.
- One primary Start Work action. No tray, autostart, shortcut, or updater.
- TDD required for production behavior. Scaffolding/config generation is exempt; establish test harness before feature code.
