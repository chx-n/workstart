# Workstart Familiar Workspace Redesign

## Status

Approved direction, awaiting written-spec review before implementation.

## Product intent

Workstart is a compact desktop utility for launching one recurring workspace. The redesign must make the primary action immediate, make application state easy to scan, and make configuration feel like a familiar desktop settings experience. It preserves all current behavior, content, security boundaries, and accessibility semantics.

The current graphite editorial treatment is an anti-reference. Decorative hairlines, underline-style buttons, tiny tracked metadata, amber-on-charcoal styling, and the permanent three-column Settings layout are removed rather than refined.

## Design direction

The chosen direction is **Familiar Workspace**, an Operate-mode interface informed by common Fluent and GNOME desktop patterns without copying either system wholesale.

- System UI typography, normal sentence casing, and readable 12-16px text replace editorial display and monospace styling.
- A cool neutral background supports one elevated application surface.
- Blue is the only interaction accent. Green and red appear only for semantic success and error states.
- Spacing, surface fills, and selection backgrounds create grouping. Decorative side borders and repeated row rules are not used.
- Panels use a 14px radius, form controls and standard buttons use 9px, and compact icon buttons use 8px.
- Motion is limited to feedback and state changes. Frequent actions do not receive entrance animation.

The interface follows the operating system light or dark preference. Light mode is designed as the default physical scene: a small utility opened during daytime desk work. Dark mode preserves the same hierarchy and does not introduce a separate visual identity.

## Main window

The main window keeps its compact footprint and three-part hierarchy.

### Header

- Workstart title and the existing one-line description remain left aligned.
- Settings becomes a conventional 36px rounded icon button with a tooltip and visible focus treatment.
- No divider sits beneath the header. The vertical gap establishes separation.

### Workflow list

- Applications live in one rounded surface with comfortable inset padding.
- Each row uses a 36px application icon, name, optional inline error copy, and a right-aligned semantic status with icon and text.
- Rows have no decorative leading border and no full-width rule after every item.
- Hover and keyboard focus use a soft neutral fill. Disabled rows reduce emphasis without becoming unreadable.
- Loading, empty, and error states occupy the same surface and use plain language plus one relevant action or instruction.

### Primary action

- Summary text sits directly above the action.
- Start Work becomes a full-width, 46px filled blue button with strong label contrast.
- Launching adds a spinner without changing button width.
- Success, retry, disabled, and failure behavior remains unchanged.

## Settings window

Settings changes from three permanent columns to a two-pane list/detail workspace.

### Top bar

- Title and description remain on the left.
- Export diagnostics is a quiet rounded secondary button on the right.
- Messages appear below the top bar as compact filled callouts, not text between rules.

### Workflow pane

- A fixed-width leading pane contains the Workflow heading, count, Add app button, and application list.
- Selected rows use a rounded filled selection background instead of an accent side border.
- Each row shows the app icon, name, and enabled state.
- Reorder controls are rounded icon buttons revealed for the selected row while remaining keyboard accessible.
- The pane maintains its own scroll area.

### Detail pane

- The selected app opens in a scrollable detail pane.
- Its header shows the application icon and name, followed by an Enable switch.
- General, Launch target, and Arguments are separate grouped sections with clear headings and generous vertical rhythm.
- Name and argument fields use complete input containers with padding, radius, focus ring, and helper or error text beneath.
- Launch target is presented as a read-only property row. Repair remains a clear secondary action when needed.
- Remove app becomes a conventional destructive button in a dedicated bottom section, away from routine editing controls.

### Add apps dialog

- Add app opens a centered modal dialog instead of dedicating a permanent third column to discovery.
- The dialog contains search, installed-app results, Add actions, Add executable, and discovery error or empty states.
- The dialog has a dim scrim, trapped visual focus, Escape and close-button dismissal, and no spring or bounce.
- Adding an item selects it, closes the dialog, and preserves current service behavior.

### Save area

- A sticky footer presents save progress or success status on the left and a filled Save changes button on the right.
- Save remains disabled while loading, saving, or validation fails.

## Visual system

### Typography

Use the platform system stack: `system-ui`, `-apple-system`, `BlinkMacSystemFont`, `Segoe UI`, and common Linux UI fallbacks. Headings use weight and spacing for hierarchy. Monospace is reserved for the launch-target value only.

### Color roles

- Window background: cool neutral gray.
- Primary surface: white in light mode and raised charcoal in dark mode.
- Secondary surface: subtle cool gray used for selected, hover, and grouped areas.
- Primary text: near-black or near-white.
- Secondary text: accessible neutral gray.
- Action accent: medium blue with separate hover and pressed values.
- Success and danger: semantic green and red, always paired with text or an icon.

Exact token values will be verified for WCAG AA contrast during implementation.

### Elevation and borders

- One subtle surface outline may distinguish a panel from the window background.
- Shadows are limited to the Add apps dialog and, if needed, the primary elevated surface.
- Inputs use a complete boundary, never an underline-only treatment.
- Repeated decorative side borders, hairline section dividers, and borders attached to text buttons are prohibited.

### Interaction and motion

- Pointer-down feedback uses `scale(0.98)` for pressable buttons.
- Hover and focus color transitions run for 120-180ms with a strong ease-out curve.
- Modal entry uses opacity and at most a 0.98-to-1 scale over 160-200ms.
- Keyboard-triggered actions do not wait for animation.
- Reduced motion removes transforms while retaining instant state and color feedback.

## Components and boundaries

The redesign stays inside the Svelte presentation layer.

- `App.svelte` retains workflow loading, launch orchestration, retry logic, status calculation, and settings-window behavior.
- `Settings.svelte` retains workflow editing, discovery, executable picking, validation, saving, diagnostics export, and icon loading. It gains local dialog state and reorganized markup only.
- `styles.css` replaces the graphite/editorial tokens and component rules with Familiar Workspace tokens and layouts.
- A small local icon wrapper may be added if an icon package is introduced. One icon family must be used throughout.
- Rust code, Tauri commands, persisted schema, platform adapters, and capability grants are unchanged.

## State behavior

The implementation must cover the complete state cycle already represented in tests:

- Main: loading, ready, disabled, launching, started, already open, partial failure, retry, empty workflow, load failure, and settings-open failure.
- Settings: loading, selected and unselected targets, validation failure, unresolved-target repair, discovery loading or failure, no discovery matches, adding a discovered app, adding an executable, reordering, saving, saved notice, and save or export failure.
- Add apps dialog: open, search, results, empty, failure, add, executable picker, close, and keyboard dismissal.

No state may fall back to unstyled browser controls or shift the primary layout unexpectedly.

## Accessibility

- Preserve semantic headings, lists, labels, buttons, alerts, status regions, and live announcements.
- Keep every action keyboard reachable with a visible focus ring.
- Add apps dialog uses appropriate dialog semantics and returns focus to its trigger after closing.
- Status never relies on color alone.
- Controls meet minimum 36px pointer targets in the compact desktop context.
- Long application names truncate visually while remaining available through the existing title text.
- Light and dark tokens must meet WCAG AA contrast.
- Reduced-motion behavior remains explicit.

## Testing and verification

Implementation follows test-driven development.

1. Update visual-contract tests to expect the Familiar Workspace identity and list/detail Settings layout.
2. Add tests for opening, searching, adding from, and dismissing the Add apps dialog.
3. Keep existing behavior tests passing without weakening assertions.
4. Run `pnpm test`, `pnpm check`, and `pnpm build`.
5. Run the Rust suite because no backend change is expected and a clean boundary check matters: `cargo test --manifest-path src-tauri/Cargo.toml`.
6. Capture both windows at their shipped sizes in light and dark modes. Inspect text clipping, scroll containment, focus, hover, disabled controls, loading, empty, and error states.
7. Run the Impeccable detector once on changed UI targets, fix mechanical findings, and complete the finish review before claiming success.

## Explicit non-goals

- No new workflow features, onboarding, named workspaces, telemetry, tray behavior, or platform integrations.
- No AIDA marketing structure, GSAP scroll effects, bento grids, glassmorphism, gradients, decorative imagery, or perpetual animation.
- No change to backend commands, configuration schema, launch security, or packaging.
- No copy rewrite beyond labels required by the Add apps interaction.

## Success criteria

- The app reads immediately as a familiar desktop utility rather than a generated editorial dashboard.
- Buttons, inputs, lists, selection, status, and destructive actions look and behave like intentional controls.
- Main action remains obvious within one glance.
- Settings is easier to scan because workflow selection and editing remain visible while app discovery appears only when requested.
- No decorative side border or underline-button pattern remains.
- Functional and accessibility tests pass, and verified light and dark captures show no clipping or layout regressions.
