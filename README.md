# Workstart

Workstart is a focused desktop launcher for one editable work workflow. It is
built with Tauri 2, Svelte 5, TypeScript, and Vite.

This repository currently contains the secure application bootstrap and test
harness. Workflow configuration, discovery, launching, and the complete user
interface are implemented in later milestones.

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

## Security model

Platform access belongs to the Rust backend. The frontend receives only
explicit Tauri capabilities, and Workstart does not include Tauri's shell
plugin. Launching will accept stored target identifiers rather than shell
command strings. Configuration and logs remain local to the machine; the app
does not send telemetry.

## License

[MIT](LICENSE)
