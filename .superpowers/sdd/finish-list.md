# Workstart v1 Finish List

## Magic moment

On a normal workday, chxnn opens Workstart on Fedora KDE and sees Antigravity IDE and Brave already configured. One press on **Start Work** launches both at once; any app already open is clearly skipped. Workstart confirms success and disappears after one second. If one app cannot start, its row explains failure and **Try Again** retries only that app.

## Experience gates

- [ ] First run contains real Antigravity and Brave defaults; valid detected targets need no setup.
- [ ] Missing/unresolved default shows clear settings repair action, never a dead disabled screen.
- [ ] Main screen fits 440×320 with two long app names without clipping critical controls.
- [ ] Start Work is obvious primary action and reachable by keyboard.
- [ ] Launching shows immediate per-app feedback; double activation cannot duplicate launches.
- [ ] Running app says **Already open** and counts as success.
- [ ] Partial failure names affected app, keeps successful state, and offers **Try Again** for failed IDs only.
- [ ] Total load/IPC failure uses plain-language recovery copy and keeps settings reachable.
- [ ] Success confirmation remains visible for one second before process exit.
- [ ] Settings first run makes installed app discovery useful; empty discovery offers executable picker.
- [ ] Invalid executable/path is shown inline and blocks save without losing edits.
- [ ] Remove, enable/disable, reorder, and argument editing work by keyboard.
- [ ] Saving settings refreshes main workflow and restores focus predictably.
- [ ] Light, dark, reduced-motion, 200% scaling, long names, empty workflow, and error states remain usable.
- [ ] Icons resolve locally; missing icon uses intentional fallback, never broken image.
- [ ] Logs omit arguments, rotate, and export only after explicit destination choice.
- [ ] Second launch focuses existing main window.
- [ ] RPM installs/menu-launches/uninstalls on Fedora KDE; GNOME and Windows remain explicitly unverified until tested there.
- [ ] README golden path commands run as written.

## Scope guard

No cloud sync, accounts, telemetry, tray, autostart, global shortcut, automatic updater, multiple workflows, UWP-only apps, macOS, or ARM64 work enters v1.
