# Task 1 Report: Bootstrap and Test Harness

## Summary

- Bootstrapped Workstart as a Tauri 2, Svelte 5, TypeScript, and Vite application managed by pnpm.
- Configured the `io.github.chxnn.workstart` application at version `0.1.0` under the MIT license.
- Defined a visible, fixed 440×320 main window and a hidden, fixed 760×600 settings window.
- Added an allowlist capability with no frontend system permissions and a production CSP that disallows remote, inline, object, form, and framing access while retaining Tauri IPC.
- Registered the Tauri single-instance plugin so later invocations restore and focus the main window.
- Added Vitest, Svelte Testing Library, jest-dom, and Rust integration-test harnesses.
- Corrected the Vite development server to port 1420 so it matches Tauri's `devUrl`, with a regression test covering the contract.
- Added repository documentation, an MIT license, and ignore rules for generated frontend and Tauri output.

## Verification

- `pnpm test`
- `pnpm check`
- `pnpm build`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `pnpm tauri build --debug --no-bundle`
- `git diff --check`

All commands above completed successfully before the Task 1 commit.

## Files

- Frontend scaffold and tests: `index.html`, `src/`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`
- JavaScript package metadata: `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`
- Tauri/Rust scaffold and tests: `src-tauri/`
- Project documentation: `README.md`, `STRUCTURE.md`, `LICENSE`
- Repository and SDD metadata: `.gitignore`, `.superpowers/sdd/`

## Remaining Risks

- The single-instance restore/focus callback compiled successfully but was not exercised with two live GUI processes in this headless verification environment.
- Windows runtime behavior was not exercised on this Fedora development host.
- The bootstrap icon is temporary; production Linux and Windows assets and package-specific settings remain intentionally deferred to Task 8.
- Workflow, platform, launch, logging, and full interface behavior remain intentionally deferred to Tasks 2–7.
