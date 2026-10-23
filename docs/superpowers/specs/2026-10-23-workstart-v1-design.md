# Workstart v1 Design

Workstart is a public MIT-licensed Tauri 2 desktop launcher for Fedora 44 and Windows 10/11 x86-64. It owns one editable workflow, preloaded with Antigravity IDE and Brave, and exposes one primary Start Work action.

The Svelte 5 frontend presents a compact launcher and a separate settings window. A Rust core owns configuration, installed-application discovery, icon lookup, best-effort running detection, direct no-shell process launch, and local rotating logs. Successful batches close after one second. Partial failures remain visible and retry only failed targets.

Configuration is machine-local, versioned JSON in the operating system app-config directory. Fedora discovery follows freedesktop desktop-entry rules. Windows discovery uses App Paths, installed-app records, and Start Menu shortcuts. Unsupported or unresolved targets use an executable picker fallback.

The interface follows the system color scheme with simulated frosted panels inside opaque standard windows. It uses one blue accent, 12px radii, short transform/opacity transitions, keyboard navigation, visible focus, reduced-motion support, and detected application icons.

Packages are an x86-64 RPM for Fedora 44 and a per-user x86-64 NSIS installer for Windows 10/11. Stable public releases require a Windows code-signing certificate, a dedicated RPM GPG key, SHA-256 checksums, and verified platform smoke tests.

Out of scope: macOS, ARM64, UWP-only launch targets, cloud sync, named workflows, telemetry, tray mode, global shortcuts, login launch, automatic updates, and native compositor blur.
