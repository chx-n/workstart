### Task 7: Settings Interface

Implement settings window UI in this worktree, based on current Workstart IPC contract. Add searchable installed-app browser, add/remove, enable/disable, reorder, executable picker fallback, safe string-array arguments, unresolved-path repair, save validation, diagnostics export, and refresh main window after save. Use injectable service fakes; no shell syntax and no arbitrary command fields. Preserve 760x600 fixed settings window, keyboard navigation/focus, light/dark/reduced motion, long names, empty/error states.

Follow TDD. Keep main launcher behavior intact. Use Tauri commands only through service boundary; if backend command is not yet present, define typed service methods and document required command names for Task 5 integration. Run frontend tests/check/build, self-review, report, commit.
