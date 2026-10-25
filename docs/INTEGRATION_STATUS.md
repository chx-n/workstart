# Integration status

Workstart implementation is merged on `integration/workstart` through Tasks 1–8.
The current Fedora verification passes frontend tests, Svelte checks, production
frontend build, Rust tests, clippy with `-D warnings`, and Tauri debug no-bundle
build.

Known environment limits:

- RPM generation needs `rpmbuild` and Fedora packaging prerequisites.
- Windows native build/runtime needs Windows 10/11; Fedora cross-build lacks
  `x86_64-w64-mingw32-windres`.
- Stable signing needs a user-owned Windows certificate and RPM GPG key.
- Executable picker and diagnostics save dialog are explicit unsupported-command
  responses until the native dialog integration is added; discovery and the
  main launch path remain available.
