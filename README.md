# Workstart

Workstart is a focused desktop launcher for one editable work workflow. It is
built with Tauri 2, Svelte 5, TypeScript, and Vite.

Workstart stores one local workflow, discovers installed applications, launches
enabled targets concurrently, skips confirmed-running apps, and retries only
failed targets. Settings edits use individual argument values; shell command
strings are never accepted.

## Supported platforms

- Fedora 44 on x86-64, using KDE Plasma or GNOME
- Windows 10 and Windows 11 on x86-64

macOS, ARM64, cloud sync, telemetry, tray mode, autostart, global shortcuts,
and automatic updates are outside the v1 scope.

## Prerequisites

- Node.js 22 or newer
- pnpm 12
- A current stable Rust toolchain
- The platform prerequisites from the
  [Tauri 2 documentation](https://v2.tauri.app/start/prerequisites/)

## Development

```sh
pnpm install
pnpm tauri dev
```

Run the local verification suite with:

```sh
pnpm test
pnpm check
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

The backend exposes only named commands: `get_workflow`, `save_workflow`,
`launch_targets`, `discover_installed_apps`, `get_app_icon`,
`pick_executable`, and `export_logs`. Platform packaging and signing remain
release-environment work; see [docs/RELEASING.md](docs/RELEASING.md).

Unsigned platform packages, signing prerequisites, and the release checklist
are documented in [docs/RELEASING.md](docs/RELEASING.md). Packaging commands do
not install or publish artifacts.

## Security model

Platform access belongs to the Rust backend. The frontend receives only
explicit Tauri capabilities, and Workstart does not include Tauri's shell
plugin. Launching will accept stored target identifiers rather than shell
command strings. Configuration and logs remain local to the machine; the app
does not send telemetry.

## License

[MIT](LICENSE)
