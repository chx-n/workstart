<script lang="ts">
  import { onMount } from 'svelte';

  import {
    tauriWorkstartService,
    type AppTarget,
    type DiscoveredApp,
    type LaunchRef,
    type WorkstartService,
    type Workflow,
  } from './lib/workstart-service';

  let { service = tauriWorkstartService }: { service?: WorkstartService } = $props();

  let apps = $state<AppTarget[]>([]);
  let discovered = $state<DiscoveredApp[]>([]);
  let selectedId = $state<string | undefined>();
  let search = $state('');
  let loading = $state(true);
  let discoveryFailed = $state(false);
  let saving = $state(false);
  let notice = $state('');
  let actionError = $state('');

  const selected = $derived(apps.find((app) => app.id === selectedId));
  const shownDiscovered = $derived(
    discovered.filter((app) => app.name.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())),
  );
  const validationErrors = $derived(validate(apps));
  const canSave = $derived(!loading && !saving && validationErrors.length === 0);

  onMount(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      const workflow = await service.getWorkflow();
      apps = workflow.apps;
      selectedId = workflow.apps[0]?.id;
      void Promise.all(workflow.apps.map(loadIcon));
    } catch {
      actionError = 'Could not load your workflow.';
    } finally {
      loading = false;
    }

    try {
      discovered = await service.discoverInstalledApps();
      void Promise.all(discovered.map(loadDiscoveredIcon));
    } catch {
      discoveryFailed = true;
    }
  }

  async function loadIcon(app: AppTarget) {
    if (app.iconDataUrl) return;
    try {
      const iconDataUrl = await service.getAppIcon(app.id);
      if (iconDataUrl) replaceApp(app.id, { iconDataUrl });
    } catch {
      // Icon lookup is optional; the local fallback remains available.
    }
  }

  async function loadDiscoveredIcon(app: DiscoveredApp) {
    if (app.iconDataUrl) return;
    try {
      const iconDataUrl = await service.getAppIcon(app.id);
      if (iconDataUrl) discovered = discovered.map((item) => item.id === app.id ? { ...item, iconDataUrl } : item);
    } catch {
      // A missing icon never prevents adding a discovered application.
    }
  }

  function replaceApp(id: string, changes: Partial<AppTarget>) {
    apps = apps.map((app) => app.id === id ? { ...app, ...changes } : app);
  }

  function addDiscovered(app: DiscoveredApp) {
    const added: AppTarget = {
      id: crypto.randomUUID(),
      name: app.name,
      launchRef: app.launchRef,
      arguments: [],
      processMatchers: app.processMatchers,
      enabled: true,
      iconDataUrl: app.iconDataUrl,
    };
    apps = [...apps, added];
    selectedId = added.id;
    void loadIcon(added);
  }

  function removeSelected() {
    if (!selected) return;
    const index = apps.findIndex((app) => app.id === selected.id);
    apps = apps.filter((app) => app.id !== selected.id);
    selectedId = apps[Math.min(index, apps.length - 1)]?.id;
  }

  function moveSelected(offset: -1 | 1) {
    if (!selected) return;
    const index = apps.findIndex((app) => app.id === selected.id);
    const destination = index + offset;
    if (destination < 0 || destination >= apps.length) return;
    const reordered = [...apps];
    [reordered[index], reordered[destination]] = [reordered[destination], reordered[index]];
    apps = reordered;
  }

  function updateSelected(changes: Partial<AppTarget>) {
    if (selected) replaceApp(selected.id, changes);
  }

  function addArgument() {
    if (selected) updateSelected({ arguments: [...selected.arguments, ''] });
  }

  function updateArgument(index: number, value: string) {
    if (!selected) return;
    const arguments_ = [...selected.arguments];
    arguments_[index] = value;
    updateSelected({ arguments: arguments_ });
  }

  function removeArgument(index: number) {
    if (selected) updateSelected({ arguments: selected.arguments.filter((_, itemIndex) => itemIndex !== index) });
  }

  async function repairSelected() {
    if (!selected) return;
    actionError = '';
    try {
      const picked = await service.pickExecutable();
      if (!picked) return;
      updateSelected({
        name: picked.name?.trim() || selected.name,
        launchRef: executableRef(picked.path),
        processMatchers: picked.processMatchers,
        iconDataUrl: picked.iconDataUrl,
        validationError: undefined,
      });
    } catch {
      actionError = 'Could not choose an executable.';
    }
  }

  async function addExecutable() {
    actionError = '';
    try {
      const picked = await service.pickExecutable();
      if (!picked) return;
      const added: AppTarget = {
        id: crypto.randomUUID(),
        name: picked.name?.trim() || 'Selected executable',
        launchRef: executableRef(picked.path),
        arguments: [],
        processMatchers: picked.processMatchers,
        enabled: true,
        iconDataUrl: picked.iconDataUrl,
      };
      apps = [...apps, added];
      selectedId = added.id;
      void loadIcon(added);
    } catch {
      actionError = 'Could not choose an executable.';
    }
  }

  async function save() {
    if (!canSave) return;
    actionError = '';
    notice = '';
    saving = true;
    try {
      const workflow: Workflow = { schemaVersion: 1, apps };
      await service.saveWorkflow(workflow);
      await service.refreshMainWindow();
      notice = 'Settings saved.';
    } catch {
      actionError = 'Could not save settings. Your changes are still here.';
    } finally {
      saving = false;
    }
  }

  async function exportDiagnostics() {
    actionError = '';
    notice = '';
    try {
      await service.exportDiagnostics();
      notice = 'Diagnostics exported.';
    } catch {
      actionError = 'Could not export diagnostics.';
    }
  }

  function executableRef(path: string): LaunchRef {
    return { kind: 'executable', value: path };
  }

  function validate(targets: AppTarget[]) {
    const errors: string[] = [];
    for (const app of targets) {
      if (!app.name.trim() || app.name.trim() !== app.name || app.name.length > 128 || /[\u0000-\u001f\u007f]/.test(app.name)) {
        errors.push(`${app.name || 'An app'} needs a trimmed name of 1–128 characters.`);
      }
      if (app.arguments.length > 64) errors.push(`${app.name} can have at most 64 arguments.`);
      if (app.arguments.some((argument) => argument.length > 4096)) errors.push('Arguments must be at most 4096 characters.');
      if (app.arguments.some((argument) => argument.includes('\0'))) errors.push('Arguments cannot contain NUL characters.');
    }
    return errors;
  }
</script>

<svelte:head><title>Workstart Settings</title></svelte:head>

<main class="settings-shell" aria-busy={loading}>
  <header class="settings-header">
    <div><h1>Settings</h1><p>Choose the apps that open with your workspace.</p></div>
    <button class="secondary-button" type="button" onclick={exportDiagnostics}>Export diagnostics</button>
  </header>

  {#if actionError}<p class="settings-message settings-message--error" role="alert">{actionError}</p>{/if}
  {#if notice}<p class="settings-message" role="status">{notice}</p>{/if}
  {#if validationErrors.length > 0}
    <div class="settings-message settings-message--error" role="alert">
      {#each validationErrors as error}<div>{error}</div>{/each}
    </div>
  {/if}

  <section class="settings-content" aria-label="Workflow settings">
    <aside class="settings-targets">
      <div class="settings-section-heading"><h2>Your workflow</h2><span>{apps.length}</span></div>
      {#if loading}
        <p class="settings-empty">Loading apps…</p>
      {:else if apps.length === 0}
        <p class="settings-empty">No apps yet. Add one from Installed apps.</p>
      {:else}
        <div class="settings-target-list" role="list" aria-label="Apps in your workflow">
          {#each apps as app, index (app.id)}
            <div class="settings-target" class:settings-target--selected={app.id === selectedId} role="listitem">
              <button class="settings-target__select" type="button" aria-pressed={app.id === selectedId} onclick={() => selectedId = app.id}>
                {#if app.iconDataUrl}<img src={app.iconDataUrl} alt="" />{:else}<span class="settings-icon" aria-hidden="true">□</span>{/if}
                <span class="settings-target__copy"><span class="settings-target__name" title={app.name}>{app.name}</span><small>{app.enabled ? 'Enabled' : 'Disabled'}</small></span>
              </button>
              <div class="settings-target__actions" aria-label={`Reorder ${app.name}`}>
                <button type="button" aria-label={`Move ${app.name} up`} disabled={index === 0} onclick={() => { selectedId = app.id; moveSelected(-1); }}>↑</button>
                <button type="button" aria-label={`Move ${app.name} down`} disabled={index === apps.length - 1} onclick={() => { selectedId = app.id; moveSelected(1); }}>↓</button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </aside>

    <section class="settings-editor" aria-label="Selected app">
      {#if selected}
        <div class="settings-section-heading"><h2>App details</h2><button class="danger-button" type="button" onclick={removeSelected}>Remove</button></div>
        <label>Name <input value={selected.name} maxlength="128" oninput={(event) => updateSelected({ name: event.currentTarget.value })} /></label>
        <label class="settings-checkbox"><input type="checkbox" checked={selected.enabled} onchange={(event) => updateSelected({ enabled: event.currentTarget.checked })} /> Enable this app</label>
        <div class="settings-launch-ref"><span>Launch target</span><code title={selected.launchRef.value}>{selected.launchRef.value}</code></div>
        {#if selected.validationError}
          <div class="repair-callout" role="alert"><span>{selected.validationError}</span><button class="secondary-button" type="button" aria-label={`Choose executable for ${selected.name}`} onclick={repairSelected}>Choose executable</button></div>
        {/if}
        <fieldset class="settings-arguments">
          <legend>Arguments</legend>
          <p>Each value is passed separately. Shell syntax is never interpreted.</p>
          {#each selected.arguments as argument, index}
            <div class="settings-argument"><label for={`argument-${selected.id}-${index}`} class="sr-only">Argument {index + 1}</label><input id={`argument-${selected.id}-${index}`} value={argument} oninput={(event) => updateArgument(index, event.currentTarget.value)} /><button type="button" aria-label={`Remove argument ${index + 1}`} onclick={() => removeArgument(index)}>Remove</button></div>
          {/each}
          <button class="secondary-button" type="button" onclick={addArgument}>Add argument</button>
        </fieldset>
      {:else}
        <p class="settings-empty">Select an app to edit its settings.</p>
      {/if}
    </section>

    <aside class="settings-discovery">
      <div class="settings-section-heading"><h2>Installed apps</h2><button class="secondary-button" type="button" onclick={addExecutable}>Add executable</button></div>
      <label class="sr-only" for="discovery-search">Search installed apps</label>
      <input id="discovery-search" placeholder="Search installed apps" bind:value={search} />
      {#if discoveryFailed}
        <p class="settings-empty" role="status">Could not load installed apps.</p>
      {:else if shownDiscovered.length === 0}
        <p class="settings-empty">No installed apps match.</p>
      {:else}
        <ul class="settings-discovery-list" aria-label="Installed apps">
          {#each shownDiscovered as app (app.id)}
            <li>{#if app.iconDataUrl}<img src={app.iconDataUrl} alt="" />{:else}<span class="settings-icon" aria-hidden="true">□</span>{/if}<span title={app.name}>{app.name}</span><button type="button" aria-label={`Add ${app.name}`} onclick={() => addDiscovered(app)}>Add</button></li>
          {/each}
        </ul>
      {/if}
    </aside>
  </section>

  <footer class="settings-footer"><span aria-live="polite">{saving ? 'Saving settings…' : ''}</span><button class="primary-button" type="button" disabled={!canSave} onclick={save}>{saving ? 'Saving…' : 'Save changes'}</button></footer>
</main>
