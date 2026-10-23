# Repository structure

```text
.
├── docs/superpowers/       Product design and implementation plan
├── src/                    Svelte frontend source and component tests
│   └── test/               Shared frontend test setup
├── src-tauri/              Tauri and Rust application
│   ├── capabilities/       Explicit frontend capability grants
│   ├── src/                Rust application entry points
│   └── tests/              Rust integration tests
├── index.html              Vite HTML entry point
├── package.json            Frontend scripts and dependencies
├── svelte.config.js        Svelte preprocessing configuration
├── tsconfig.json           TypeScript checking configuration
└── vite.config.ts          Vite and Vitest configuration
```

## Boundaries

The Svelte frontend owns presentation and user interaction. It must not access
the operating system directly. The Rust crate owns platform integration,
persistence, process inspection, and launching through narrow Tauri commands.

`src-tauri/capabilities/default.json` is the allowlist boundary between those
layers. Dependencies that can execute arbitrary shell command strings are not
permitted.
