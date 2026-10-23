# Workstart v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Build and package Workstart, a secure cross-platform launcher with an editable application workflow.

**Architecture:** Svelte 5 + TypeScript + Vite render two Tauri windows. Rust commands own all platform access and persist a versioned local workflow. Platform adapters isolate Fedora and Windows discovery, process inspection, icons, and launches.

**Tech Stack:** Tauri 2, Rust, Svelte 5, TypeScript, Vite, pnpm, Vitest, Svelte Testing Library.

## Global Constraints

- Identifier: `io.github.chxnn.workstart`; version: `0.1.0`; license: MIT.
- Fedora 44 KDE/GNOME and Windows 10/11 x86-64 only.
- Never invoke a shell or accept shell command strings.
- Launch stored target IDs only; backend resolves executable configuration.
- Local machine configuration and logs only; no network telemetry.
- Main window 440x320; settings window 760x600.
- One primary Start Work action. No tray, autostart, shortcut, or updater.
- TDD required for production behavior.

---

### Task 1: Bootstrap and Test Harness

Create a Tauri 2 + Svelte 5 + TypeScript + Vite project using pnpm. Configure package identity, main/settings windows, restrictive capabilities/CSP, single-instance behavior, Vitest, Svelte Testing Library, Rust test support, MIT license, README, and structure documentation. Do not add shell plugin. Verify frontend tests, Rust tests, typecheck, and production frontend build. Commit the task.

### Task 2: Domain Model and Configuration

Use TDD to implement WorkflowConfig, AppTarget, LaunchRef, ProcessMatcher, RunningState, LaunchOutcome, validation, default Antigravity/Brave targets, versioned JSON loading, corrupt-file recovery, and atomic save. Expose get_workflow/save_workflow through narrow Tauri commands. Ensure duplicate IDs, invalid names, excessive argument counts/lengths, and unresolved paths are rejected. Commit the task.

### Task 3: Linux Platform Adapter

Use TDD and fixtures to implement XDG desktop-entry discovery, user precedence, Hidden/NoDisplay/TryExec handling, localized names, Exec parsing and field-code removal, icon lookup, default target detection, direct no-shell launch references, and gtk-launch fallback only for DBus-only entries without custom arguments. Commit the task.

### Task 4: Windows Platform Adapter

Use cfg-gated TDD and fixtures to implement App Paths and uninstall-record mapping, Start Menu shortcut abstraction, executable fallback selection contract, icon extraction abstraction, default Brave/Antigravity candidates, and exclusion of UWP-only entries. Windows-specific code must cross-check compile where possible and run natively later. Commit the task.

### Task 5: Launch Orchestration and Logs

Use TDD to implement canonical-path and known-alias running checks, Unknown fallback launch behavior, concurrent target launch, independent outcomes, failed-only retry through target IDs, redacted structured logs, seven 1MB rotating files, and log export. Expose discover_installed_apps, get_app_icon, launch_targets, and export_logs. Commit the task.

### Task 6: Svelte Main Interface

Use TDD to implement workflow loading, app status rows, one Start Work button, Ready/Launching/Already open/Started/Failed states, empty/invalid disabled state, failed-only retry, one-second successful close, settings-window opening, keyboard focus, reduced motion, system theme, and responsive overflow within the fixed window. Commit the task.

### Task 7: Settings Interface

Use TDD to implement the separate settings window, searchable discovery list, add/remove/enable/reorder, executable picker fallback, argument-array editing, validation errors, save and main-window refresh, lazy real icons with fallback, and log export. Commit the task.

### Task 8: Brand Assets and Packaging

Generate an original Workstart icon, inspect it, copy the selected source into the repository, and derive required Linux PNG and Windows ICO assets. Configure Fedora RPM and per-user Windows NSIS bundles with WebView2 downloadBootstrapper. Add unsigned development packaging scripts and signing documentation without generating or storing secrets. Commit the task.

### Task 9: Verification and Release Documentation

Run full frontend/Rust/typecheck/build suites, Fedora KDE runtime smoke tests, RPM content/dependency checks, and any available Windows cross-check. Add manual GNOME VM, Windows 10/11, signing, checksum, and GitHub Release checklists. Perform final whole-branch review and resolve all Critical/Important findings. Commit the task.
