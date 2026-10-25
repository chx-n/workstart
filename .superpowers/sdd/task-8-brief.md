### Task 8: Production Assets and Packaging Configuration

Implement production icon derivatives and release packaging configuration. Work only in this task worktree; avoid frontend and Rust domain logic.

Source icon: `src-tauri/icons/icon.png` is the original generated transparent RGBA Workstart mark. Preserve it and generate required Linux PNG sizes and Windows ICO (and any Tauri-mandated icon assets) using deterministic local tooling or Tauri's icon command. Verify alpha, dimensions, and small-size readability as far as command-line inspection permits.

Configure Tauri bundle metadata for:

- identifier `io.github.chxnn.workstart`, product/version/license/publisher metadata.
- Fedora x86-64 RPM bundle with correct desktop categories, app icon, and no forbidden tray/autostart/updater integration.
- Windows x86-64 NSIS per-user installer, Start menu entry, WebView2 `downloadBootstrapper`, no admin requirement, no MSI requirement.
- Keep signing credentials out of the repository. Document expected Windows certificate and RPM GPG signing steps as release prerequisites, not secrets/configured fake keys.

Add packaging-focused validation tests/scripts where practical and release documentation/checklist. Do not publish, create GitHub resources, install packages, or generate signing keys. Run relevant config validation, frontend checks, Rust tests, and debug no-bundle build. Self-review, write `.superpowers/sdd/task-8-report.md`, commit, and return SHA.
