# Task 7 Report: Settings Interface

## Delivered

- Added a dedicated `?window=settings` route with a fixed-window-friendly,
  keyboard-operable settings editor. It supports a searchable installed-app
  list, adding discovered apps or a picked executable, selection, removal,
  enable/disable, button-based reordering, long-name truncation, empty and
  recoverable error states, light/dark colors, and reduced-motion inheritance.
- Added an app detail editor with a safe string-array argument editor. Values
  are displayed and submitted as individual strings; no command or shell field
  exists. Local validation rejects invalid names, NUL-containing arguments,
  oversized arguments, and more than 64 arguments before save.
- Added unresolved-target repair through the executable picker, lazy icon
  loading with local fallbacks, diagnostics export, save feedback, and an
  event-based main-window workflow refresh after a successful save.
- Extended the injectable `WorkstartService` boundary; tests use fakes rather
  than Tauri APIs directly.

## Task 5 Backend Handoff

The frontend calls only these fixed command names:

| Service method | Command | Payload / result |
| --- | --- | --- |
| `discoverInstalledApps` | `discover_installed_apps` | `DiscoveredApp[]` |
| `getAppIcon` | `get_app_icon` | `{ appId }` / optional data URL |
| `pickExecutable` | `pick_executable` | optional picked executable object |
| `saveWorkflow` | `save_workflow` | `{ workflow: Workflow }` |
| `exportDiagnostics` | `export_logs` | no payload |

`save_workflow` already exists, but its current Rust argument is the disk
`WorkflowConfig` shape while the settings window operates on the current
camel-case frontend `Workflow` view. Task 5 integration must provide an
explicit boundary mapping (or make the command accept this narrow editor view)
without allowing a config path, shell text, or arbitrary commands. The icon
command must resolve only backend-issued discovery IDs/workflow target IDs.
`pick_executable` must return a canonical path and backend-derived process
matchers, never user-authored command text.

## TDD Evidence

- RED: settings/service tests first failed because `Settings.svelte` and the
  typed service methods did not exist.
- RED: the executable-fallback test then failed because no accessible `Add
  executable` action existed.
- GREEN: 6 settings interaction tests and 5 service contract tests cover
  discovery search/add, safe arguments and validation, unresolved repair,
  picker fallback, reorder/save/main refresh, and recoverable failures.

## Verification

- `pnpm test` — 4 files passed, 24 tests passed.
- `pnpm check` — 0 errors and 0 warnings.
- `pnpm build` — production build completed successfully.
- `git diff --check` — clean.
