# Workstart context snapshot

## Goal

Replace the graphite editorial interface with a familiar cross-platform desktop productivity design, then verify every existing workflow and state.

## Approved direction

- Familiar Workspace: Fluent 2 and GNOME-informed product UI.
- Soft neutral surfaces, system light/dark theme, one restrained blue accent.
- Proper filled and outlined controls, comfortable margins, readable typography, and rounded grouped surfaces.
- No editorial hairlines, underline-style buttons, tiny tracked mono labels, decorative side borders, bento grids, gradients, glass, or ornamental motion.
- Main window remains focused on the app list and one prominent Start Work action.
- Settings becomes a two-pane list/detail workspace. Installed-app discovery moves into an Add apps surface instead of a permanent third column.

## Product and technical constraints

- Preserve all behavior, copy, test semantics, Tauri commands, and security boundaries.
- Svelte 5 frontend owns presentation; Rust owns OS access and persistence.
- Fedora 44 and Windows 10/11 x86-64 remain supported.
- System theme, visible focus, reduced motion, and readable status text remain required.
- No shell plugin or shell command strings; local config and logs; no telemetry.

## Current state

- Working tree was clean before redesign planning.
- Current visual implementation is in `src/App.svelte`, `src/Settings.svelte`, and `src/styles.css`.
- Existing tests cover primary, loading, empty, error, launch, retry, settings, discovery, validation, and save states.
- `PRODUCT.md` now records durable product truth.

## Next gate

Write and review the redesign specification. After approval, create an implementation plan, implement with tests, capture both windows in light and dark modes, run the Impeccable detector and finish review, then document the shipped visual system.
