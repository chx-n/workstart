# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

People using Fedora 44 or Windows 10/11 who start the same small set of work applications together and want one fast, predictable launch action.

## Product Purpose

Workstart owns one editable local workflow. It discovers installed applications, shows their launch readiness, opens enabled targets concurrently, skips confirmed-running apps, and retries only failed targets. Success means the user can begin work without manually opening each application.

## Positioning

Workstart is a focused local launcher for one recurring workspace, not a general app launcher, automation platform, or workflow catalog.

## Operating Context

The primary window is a compact, frequently used desktop utility. The separate Settings window is used less often to add, remove, reorder, enable, repair, and configure launch targets. The product runs inside standard Fedora and Windows desktop windows and follows the operating system color preference.

## Capabilities and Constraints

- Tauri 2, Svelte 5, TypeScript, Vite, and a Rust backend.
- One editable workflow with one primary Start Work action.
- Machine-local configuration and logs; no telemetry or cloud sync.
- Platform access stays behind named Tauri commands.
- No shell plugin and no shell command strings.
- Fedora 44 and Windows 10/11 on x86-64 are supported.
- macOS, ARM64, tray mode, autostart, global shortcuts, and automatic updates are outside v1 scope.
- Existing user-visible behavior, labels, validation, accessibility semantics, and security boundaries remain intact during the redesign.

## Brand Commitments

The product name is Workstart. Voice is concise, direct, and functional. The existing application icon remains the identity asset. The replacement interface must look like a familiar desktop productivity app and must avoid editorial hairlines, underline-style buttons, tiny tracked metadata, decorative side borders, generic dashboard cards, and ornamental effects.

## Evidence on Hand

- Product behavior and constraints: `README.md` and `docs/superpowers/specs/2026-10-23-workstart-v1-design.md`.
- Current frontend and states: `src/App.svelte`, `src/Settings.svelte`, and their tests.
- Current visual implementation: `src/styles.css`.
- Existing icon assets: `src-tauri/icons/`.
- No customer claims, telemetry, benchmarks, or external brand assets are available and none may be fabricated.

## Product Principles

- Put the recurring launch action first.
- Make state legible without demanding attention.
- Keep configuration safe, local, and understandable.
- Use familiar desktop patterns before inventing new interaction models.
- Preserve keyboard access, reduced motion, and system-theme support.

## Accessibility & Inclusion

Maintain semantic controls, visible keyboard focus, WCAG AA contrast, reduced-motion behavior, readable status text, and layouts that tolerate longer labels without clipping critical actions.
