# Familiar Workspace Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Workstart's graphite editorial interface with the approved Familiar Workspace design while preserving every launch, editing, validation, and security behavior.

**Architecture:** Keep all service and state logic in the existing Svelte components. Change only presentation markup, local Add apps dialog state, semantic design tokens, component styling, icon rendering, and the Settings window size. Use the approved Grouped Settings comp as the spatial reference and preserve the Rust boundary unchanged.

**Tech Stack:** Svelte 5, TypeScript 5.9, Vite 7, Vitest, Testing Library, Tauri 2, CSS custom properties, `phosphor-icons-svelte` 2.1.x.

## Global Constraints

- Preserve all current backend commands, persisted schema, launch behavior, validation rules, copy, and security boundaries.
- Main window remains `440 × 320`; Settings becomes `820 × 620`.
- Follow system light and dark preference with one blue interaction accent.
- No editorial hairlines, underline-style buttons, tiny tracked metadata, decorative side borders, bento grids, gradients, glass, or ornamental motion.
- Use full-row selection fills, complete input containers, and rounded real buttons.
- Keep visible focus, semantic status text, reduced motion, and WCAG AA contrast.
- Use TDD for every behavior or contract change.
- Approved comp: `.impeccable/mocks/familiar-workspace-grouped-settings.png`.

---

## File map

- `package.json`, `pnpm-lock.yaml`: add the Svelte Phosphor icon library.
- `src/App.test.ts`: lock the new main-surface design contract.
- `src/App.svelte`: replace inline icons and graphite contract attributes; keep all launch logic unchanged.
- `src/Settings.test.ts`: lock list/detail layout and Add apps dialog behavior.
- `src/Settings.svelte`: reorganize into workflow/detail panes and add local dialog open/close behavior.
- `src/styles.css`: replace graphite/editorial tokens and rules with Familiar Workspace tokens and components.
- `src-tauri/tauri.conf.json`: increase only the Settings window dimensions.
- `docs/superpowers/specs/2026-11-01-workstart-familiar-workspace-redesign.md`: mark the approved spec implemented after verification.
- `DESIGN.md`: document the shipped visual system after finish review.

### Task 1: Lock dependencies and visual contracts

**Files:**
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`
- Modify: `src/App.test.ts`
- Modify: `src/Settings.test.ts`

**Interfaces:**
- Consumes: existing Svelte render tests and semantic `main` elements.
- Produces: `data-visual-theme="familiar-workspace"` and `data-layout="launcher"` / `data-layout="list-detail-settings"` contracts.

- [ ] **Step 1: Change tests first**

Replace the two graphite/editorial contract tests in `src/App.test.ts` with:

```ts
it('uses the familiar workspace visual contract', async () => {
  render(App, { props: { service: service([target()]) } });

  expect(await screen.findByRole('main')).toHaveAttribute('data-visual-theme', 'familiar-workspace');
  expect(screen.getByRole('main')).toHaveAttribute('data-layout', 'launcher');
});
```

Replace the corresponding tests in `src/Settings.test.ts` with:

```ts
it('uses the familiar list-detail settings contract', async () => {
  render(Settings, { props: { service: service() } });

  expect(await screen.findByRole('main')).toHaveAttribute('data-visual-theme', 'familiar-workspace');
  expect(screen.getByRole('main')).toHaveAttribute('data-layout', 'list-detail-settings');
});
```

- [ ] **Step 2: Verify RED**

Run:

```bash
pnpm test -- src/App.test.ts src/Settings.test.ts
```

Expected: both new assertions fail because components still expose `graphite` and `editorial-command-sheet`.

- [ ] **Step 3: Update component contracts minimally**

In `src/App.svelte`:

```svelte
<main class="app-shell" data-visual-theme="familiar-workspace" data-layout="launcher">
```

In `src/Settings.svelte`:

```svelte
<main class="settings-shell" data-visual-theme="familiar-workspace" data-layout="list-detail-settings" aria-busy={loading}>
```

- [ ] **Step 4: Verify GREEN**

Run the same targeted test command. Expected: contract tests pass and existing behavior tests remain green.

- [ ] **Step 5: Add icon dependency**

Run:

```bash
pnpm add -D phosphor-icons-svelte@2.1.0
```

Expected: `package.json` and `pnpm-lock.yaml` record the package without changing existing dependency versions.

- [ ] **Step 6: Commit**

```bash
git add package.json pnpm-lock.yaml src/App.test.ts src/Settings.test.ts src/App.svelte src/Settings.svelte
git commit -m "test: lock familiar workspace contracts"
```

### Task 2: Build the Familiar Workspace main launcher

**Files:**
- Modify: `src/App.svelte`
- Modify: `src/App.test.ts`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes: existing `TargetStatus`, `statuses`, `itemErrors`, `summary`, and `startWork()`.
- Produces: semantic status icon mapping and approved launcher hierarchy.

- [ ] **Step 1: Add failing launcher-structure assertions**

Append to the existing ready-state test in `src/App.test.ts`:

```ts
expect(screen.getByRole('list', { name: 'Apps in your workflow' })).toHaveClass('target-list');
expect(screen.getByRole('button', { name: 'Start Work' })).toHaveClass('primary-button', 'primary-button--full');
expect(screen.getAllByLabelText('Ready')).toHaveLength(2);
```

- [ ] **Step 2: Verify RED**

Run:

```bash
pnpm test -- src/App.test.ts
```

Expected: failure because the button lacks `primary-button--full` and status elements lack accessible icon labels.

- [ ] **Step 3: Import one icon family and map statuses**

At the top of `src/App.svelte`, add regular/fill Phosphor icon imports for gear, check circle, warning circle, prohibition, and spinner gap. Add:

```ts
function statusLabel(status: TargetStatus) {
  return status;
}
```

Render each status as icon plus text, with the icon span carrying `aria-label={statusLabel(statuses[app.id])}` and the visible label hidden from duplicate accessibility output with `aria-hidden="true"`.

- [ ] **Step 4: Replace only main-window markup**

Keep the existing branches and handlers. Recompose them as:

```svelte
<header class="app-header">
  <div class="app-heading">
    <h1>Workstart</h1>
    <p>Start your workspace in one click.</p>
  </div>
  <button class="icon-button" type="button" aria-label="Open settings" title="Settings" onclick={openSettings}>
    <IconGearRegular aria-hidden="true" />
  </button>
</header>

<section class="workflow-surface" aria-label="Apps in your workflow">
  {#if loading}
    <div class="state-panel" role="status"><span class="state-spinner" aria-hidden="true"></span>Loading apps…</div>
  {:else if loadError}
    <div class="state-panel state-panel--error" role="alert"><strong>Could not load your workflow.</strong><span>Open Settings and try again.</span></div>
  {:else if targets.length === 0}
    <div class="state-panel"><strong>No apps yet</strong><span>Add apps in Settings to build your workflow.</span></div>
  {:else}
    <ul class="target-list" aria-label="Apps in your workflow">
      {#each targets as app}
        <li class:target-failed={statuses[app.id] === 'Failed'}>
          <span class="app-icon" aria-hidden="true"><IconAppWindowRegular /></span>
          <span class="target-copy"><strong>{app.name}</strong>{#if itemErrors[app.id]}<small>{itemErrors[app.id]}</small>{/if}</span>
          <span class="target-status target-status--{statuses[app.id].toLowerCase()}">
            <span class="status-icon" aria-label={statusLabel(statuses[app.id])}>{statusIcon(statuses[app.id])}</span>
            <span aria-hidden="true">{statuses[app.id]}</span>
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<footer class="action-area">
  {#if settingsError}<p class="settings-error" role="alert">Could not open Settings.</p>{/if}
  <p class="summary" role="status" aria-live="polite">{summary}</p>
  <button class="primary-button primary-button--full" type="button" disabled={!canLaunch} onclick={startWork}>
    {#if launching}<span class="button-spinner" aria-hidden="true"></span>{/if}
    {complete ? 'Work started' : buttonLabel}
  </button>
</footer>
```

- [ ] **Step 5: Replace main-window CSS with approved tokens and layout**

Define semantic tokens in `:root` for `--window`, `--surface`, `--surface-subtle`, `--surface-selected`, `--text`, `--text-secondary`, `--border`, `--accent`, semantic status colors, radii, shadow, and ease. Implement:

```css
.app-shell {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 14px;
  width: 100%;
  height: 100%;
  padding: 18px;
  background: var(--window);
}

.workflow-surface {
  min-height: 0;
  overflow: hidden;
  padding: 8px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-surface);
}

.target-list li {
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  min-height: 54px;
  padding: 8px 10px;
  border: 0;
  border-radius: var(--radius-control);
}

.primary-button--full {
  width: 100%;
  min-height: 46px;
}
```

Use `@media (prefers-color-scheme: dark)` for dark tokens and keep the existing reduced-motion override.

- [ ] **Step 6: Verify GREEN**

Run:

```bash
pnpm test -- src/App.test.ts
pnpm check
```

Expected: all main-window tests pass; Svelte check reports no errors.

- [ ] **Step 7: Commit**

```bash
git add src/App.svelte src/App.test.ts src/styles.css
git commit -m "feat: redesign main launcher surface"
```

### Task 3: Add the test-driven Add apps dialog

**Files:**
- Modify: `src/Settings.test.ts`
- Modify: `src/Settings.svelte`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes: existing `shownDiscovered`, `addDiscovered()`, `addExecutable()`, and discovery states.
- Produces: `showAddApps: boolean`, `openAddApps()`, `closeAddApps()`, and dialog semantics named `Add apps`.

- [ ] **Step 1: Add failing dialog tests**

Add these tests to `src/Settings.test.ts`:

```ts
it('opens and dismisses the Add apps dialog', async () => {
  render(Settings, { props: { service: service() } });
  await screen.findByText('Antigravity IDE');

  expect(screen.queryByRole('dialog', { name: 'Add apps' })).not.toBeInTheDocument();
  await fireEvent.click(screen.getByRole('button', { name: 'Add app' }));
  expect(screen.getByRole('dialog', { name: 'Add apps' })).toBeInTheDocument();

  await fireEvent.click(screen.getByRole('button', { name: 'Close Add apps' }));
  expect(screen.queryByRole('dialog', { name: 'Add apps' })).not.toBeInTheDocument();
});

it('adds a discovered app from the dialog and closes it', async () => {
  render(Settings, { props: { service: service() } });
  await screen.findByText('Antigravity IDE');
  await fireEvent.click(screen.getByRole('button', { name: 'Add app' }));
  await fireEvent.input(screen.getByLabelText('Search installed apps'), { target: { value: 'brave' } });
  await fireEvent.click(screen.getByRole('button', { name: 'Add Brave Browser' }));

  expect(await screen.findByText('Brave Browser', { selector: '.settings-target__name' })).toBeInTheDocument();
  expect(screen.queryByRole('dialog', { name: 'Add apps' })).not.toBeInTheDocument();
});
```

Update the existing filter/add test so it opens the dialog before finding `Installed apps`.

- [ ] **Step 2: Verify RED**

Run:

```bash
pnpm test -- src/Settings.test.ts
```

Expected: `Add app` and dialog queries fail because the third discovery column is still permanent.

- [ ] **Step 3: Add minimal dialog state**

In `src/Settings.svelte`:

```ts
let showAddApps = $state(false);

function openAddApps() {
  search = '';
  showAddApps = true;
}

function closeAddApps() {
  showAddApps = false;
}
```

At the end of `addDiscovered()`, set `showAddApps = false`. At the successful end of `addExecutable()`, also set it false.

- [ ] **Step 4: Render semantic dialog**

Move the existing discovery markup into:

```svelte
{#if showAddApps}
  <div class="dialog-backdrop" role="presentation" onclick={(event) => event.target === event.currentTarget && closeAddApps()}>
    <section class="add-apps-dialog" role="dialog" aria-modal="true" aria-labelledby="add-apps-title">
      <header class="dialog-header">
        <div><h2 id="add-apps-title">Add apps</h2><p>Choose an installed app or browse for an executable.</p></div>
        <button class="icon-button" type="button" aria-label="Close Add apps" onclick={closeAddApps}><IconXRegular aria-hidden="true" /></button>
      </header>
      <label for="discovery-search">Search installed apps</label>
      <input id="discovery-search" placeholder="Search installed apps" bind:value={search} />
      {#if discoveryFailed}
        <p class="settings-empty" role="status">Could not load installed apps.</p>
      {:else if shownDiscovered.length === 0}
        <p class="settings-empty">No installed apps match.</p>
      {:else}
        <ul class="settings-discovery-list" aria-label="Installed apps">
          {#each shownDiscovered as app (app.id)}
            <li>
              {#if app.iconDataUrl}<img src={app.iconDataUrl} alt="" />{:else}<span class="settings-icon" aria-hidden="true"><IconAppWindowRegular /></span>{/if}
              <span title={app.name}>{app.name}</span>
              <button class="quiet-button" type="button" aria-label={`Add ${app.name}`} onclick={() => addDiscovered(app)}>Add</button>
            </li>
          {/each}
        </ul>
      {/if}
      <button class="secondary-button" type="button" onclick={addExecutable}>Add executable</button>
    </section>
  </div>
{/if}
```

Add a document keydown handler that calls `closeAddApps()` for Escape only while the dialog is open. Return focus to the Add app trigger after close using a bound button reference and `tick()`.

- [ ] **Step 5: Verify GREEN**

Run the targeted Settings tests. Expected: open, filter, add, close, executable-picker, and existing save tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/Settings.svelte src/Settings.test.ts src/styles.css
git commit -m "feat: move app discovery into dialog"
```

### Task 4: Build the two-pane Grouped Settings layout

**Files:**
- Modify: `src/Settings.svelte`
- Modify: `src/Settings.test.ts`
- Modify: `src/styles.css`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: selected app state and every existing edit/reorder/remove/save handler.
- Produces: leading workflow pane, grouped detail pane, switch control, sticky save footer.

- [ ] **Step 1: Add failing layout assertions**

In the first Settings rendering test, assert:

```ts
expect(screen.getByRole('complementary', { name: 'Workflow' })).toBeInTheDocument();
expect(screen.getByRole('region', { name: 'App details' })).toBeInTheDocument();
expect(screen.getByRole('button', { name: 'Add app' })).toBeInTheDocument();
expect(screen.getByRole('button', { name: 'Remove Antigravity IDE' })).toBeInTheDocument();
```

- [ ] **Step 2: Verify RED**

Run `pnpm test -- src/Settings.test.ts`. Expected: landmark and button-name assertions fail.

- [ ] **Step 3: Recompose existing controls without changing handlers**

Use this structural hierarchy:

```svelte
<section class="settings-content" aria-label="Workflow settings">
  <aside class="workflow-pane" aria-label="Workflow">
    <header class="pane-header">
      <div><h2>Workflow</h2><span>{apps.length}</span></div>
      <button class="primary-button primary-button--compact" type="button" bind:this={addAppButton} onclick={openAddApps}>Add app</button>
    </header>
    {#if loading}
      <p class="settings-empty">Loading apps…</p>
    {:else if apps.length === 0}
      <p class="settings-empty">No apps yet. Add one to build your workflow.</p>
    {:else}
      <div class="settings-target-list" role="list" aria-label="Apps in your workflow">
        {#each apps as app, index (app.id)}
          <div class="settings-target" class:settings-target--selected={app.id === selectedId} role="listitem">
            <button class="settings-target__select" type="button" aria-pressed={app.id === selectedId} onclick={() => selectedId = app.id}>
              {#if app.iconDataUrl}<img src={app.iconDataUrl} alt="" />{:else}<span class="settings-icon" aria-hidden="true"><IconAppWindowRegular /></span>{/if}
              <span class="settings-target__copy"><span class="settings-target__name" title={app.name}>{app.name}</span><small>{app.enabled ? 'Enabled' : 'Disabled'}</small></span>
            </button>
            {#if app.id === selectedId}
              <div class="settings-target__actions" aria-label={`Reorder ${app.name}`}>
                <button type="button" aria-label={`Move ${app.name} up`} disabled={index === 0} onclick={() => moveSelected(-1)}><IconArrowUpRegular /></button>
                <button type="button" aria-label={`Move ${app.name} down`} disabled={index === apps.length - 1} onclick={() => moveSelected(1)}><IconArrowDownRegular /></button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </aside>

  <section class="detail-pane" aria-label="App details">
    {#if selected}
      <header class="detail-identity">
        {#if selected.iconDataUrl}<img src={selected.iconDataUrl} alt="" />{:else}<span class="settings-icon" aria-hidden="true"><IconAppWindowRegular /></span>{/if}
        <h2>{selected.name}</h2>
        <label class="switch-control"><span class="sr-only">Enable this app</span><input type="checkbox" checked={selected.enabled} onchange={(event) => updateSelected({ enabled: event.currentTarget.checked })} /><span aria-hidden="true"></span></label>
      </header>
      <section class="setting-group"><h3>General</h3><label class="setting-row"><span>Name</span><input value={selected.name} maxlength="128" oninput={(event) => updateSelected({ name: event.currentTarget.value })} /></label></section>
      <section class="setting-group"><h3>Launch</h3><div class="setting-row"><span>Launch target</span><code title={selected.launchRef.value}>{selected.launchRef.value}</code></div></section>
      <section class="setting-group"><h3>Arguments</h3><p>Each value is passed separately. Shell syntax is never interpreted.</p>{#each selected.arguments as argument, index}<div class="settings-argument"><label for={`argument-${selected.id}-${index}`} class="sr-only">Argument {index + 1}</label><input id={`argument-${selected.id}-${index}`} value={argument} oninput={(event) => updateArgument(index, event.currentTarget.value)} /><button type="button" aria-label={`Remove argument ${index + 1}`} onclick={() => removeArgument(index)}><IconXRegular /></button></div>{/each}<button class="secondary-button" type="button" onclick={addArgument}>Add argument</button></section>
      <button class="danger-row" type="button" aria-label={`Remove ${selected.name}`} onclick={removeSelected}>Remove app</button>
    {:else}
      <p class="settings-empty">Select an app to edit its settings.</p>
    {/if}
  </section>
</section>
```

Keep the existing up/down actions and accessible names. Style the enable checkbox as a switch while retaining native checkbox semantics.

- [ ] **Step 4: Replace Settings CSS**

Implement:

```css
.settings-shell {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  gap: 16px;
  width: 100%;
  height: 100%;
  padding: 20px;
  background: var(--window);
}

.settings-content {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-surface);
}

.workflow-pane {
  min-width: 0;
  padding: 16px 12px;
  overflow: hidden;
  background: var(--surface-subtle);
}

.detail-pane {
  min-width: 0;
  padding: 20px 28px 28px;
  overflow: auto;
}

.settings-target--selected {
  background: var(--surface-selected);
  box-shadow: none;
}

.setting-row {
  display: grid;
  grid-template-columns: minmax(120px, 0.7fr) minmax(180px, 1.3fr);
  align-items: center;
  gap: 20px;
  min-height: 58px;
  padding: 10px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-control);
}

.settings-footer {
  display: flex;
  min-height: 48px;
  align-items: center;
  justify-content: space-between;
}
```

At widths below 700px, keep two panes but reduce the leading pane to 210px and detail padding to 18px. The shipped window width is 820px.

- [ ] **Step 5: Update Settings window dimensions**

In `src-tauri/tauri.conf.json`, change only the Settings entry:

```json
"width": 820,
"height": 620
```

- [ ] **Step 6: Verify GREEN**

Run:

```bash
pnpm test -- src/Settings.test.ts
pnpm check
pnpm build
```

Expected: all settings tests pass, Svelte reports no errors, and Vite builds successfully.

- [ ] **Step 7: Commit**

```bash
git add src/Settings.svelte src/Settings.test.ts src/styles.css src-tauri/tauri.conf.json
git commit -m "feat: redesign settings as grouped list detail"
```

### Task 5: Verify, inspect, and document the shipped system

**Files:**
- Modify: `docs/superpowers/specs/2026-11-01-workstart-familiar-workspace-redesign.md`
- Create: `DESIGN.md`
- Create: `.impeccable/review/main-light.png`
- Create: `.impeccable/review/main-dark.png`
- Create: `.impeccable/review/settings-light.png`
- Create: `.impeccable/review/settings-dark.png`

**Interfaces:**
- Consumes: completed frontend implementation and approved comp.
- Produces: verification evidence, detector results, finish-review verdict, and durable design documentation.

- [ ] **Step 1: Run the complete automated suite**

```bash
pnpm test
pnpm check
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: every command exits 0 with no warnings that indicate broken behavior.

- [ ] **Step 2: Capture and inspect shipped windows**

Run the app or browser harness at the shipped dimensions. Capture main and Settings in light and dark modes into `.impeccable/review/`. Inspect clipping, scroll containment, focus, hover, loading, empty, error, disabled, dialog, and destructive states in one batched pass.

- [ ] **Step 3: Run the Impeccable detector once**

```bash
node .agents/skills/design/impeccable/scripts/detect.mjs --json src/App.svelte src/Settings.svelte src/styles.css
```

Expected: no unresolved mechanical finding that conflicts with the approved spec.

- [ ] **Step 4: Complete finish review**

Send original request, spec, approved comp, changed targets, detector output, and four validated screenshots to `impeccable-finish-reviewer`. Apply at most the bounded fix rounds required by its verdict.

- [ ] **Step 5: Document actual shipped design**

Run the Impeccable documenter with project root, artifact targets, direction contract, `PRODUCT.md`, and the final implementation. Write `DESIGN.md` from the built result, not from intention.

- [ ] **Step 6: Mark spec implemented and commit**

Change the spec status to:

```markdown
Implemented and verified.
```

Then commit:

```bash
git add DESIGN.md docs/superpowers/specs/2026-11-01-workstart-familiar-workspace-redesign.md .impeccable
git commit -m "docs: record familiar workspace design system"
```

## Plan self-review

- Spec coverage: main launcher, two-pane Settings, Add apps dialog, tokens, icons, system theme, states, accessibility, window dimensions, tests, visual inspection, finish review, and documentation all map to explicit tasks.
- Deferred-work scan: no unfinished markers or unspecified error-handling step remains.
- Type consistency: the plan reuses existing `AppTarget`, `TargetStatus`, handlers, and service contracts. New local names are consistently `showAddApps`, `openAddApps`, and `closeAddApps`.
