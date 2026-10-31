<script lang="ts">
  import IconAppWindowRegular from 'phosphor-icons-svelte/IconAppWindowRegular.svelte';
  import IconArrowDownRegular from 'phosphor-icons-svelte/IconArrowDownRegular.svelte';
  import IconArrowUpRegular from 'phosphor-icons-svelte/IconArrowUpRegular.svelte';
  import IconDownloadSimpleRegular from 'phosphor-icons-svelte/IconDownloadSimpleRegular.svelte';
  import IconMagnifyingGlassRegular from 'phosphor-icons-svelte/IconMagnifyingGlassRegular.svelte';
  import IconPlusRegular from 'phosphor-icons-svelte/IconPlusRegular.svelte';
  import IconXRegular from 'phosphor-icons-svelte/IconXRegular.svelte';
  import { onMount, tick } from 'svelte';

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
  let showAddApps = $state(false);
  let addAppButton: HTMLButtonElement;
  let addAppsDialog = $state<HTMLElement>();

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
    void closeAddApps();
  }

  async function openAddApps() {
    search = '';
    showAddApps = true;
    await tick();
    addAppsDialog?.querySelector<HTMLInputElement>('#discovery-search')?.focus();
  }

  async function closeAddApps() {
    showAddApps = false;
    await tick();
    addAppButton?.focus();
  }

  function handleDialogKeydown(event: KeyboardEvent) {
    if (!showAddApps) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      void closeAddApps();
      return;
    }
    if (event.key !== 'Tab') return;

    const focusable = Array.from(
      addAppsDialog?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled)') ?? [],
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
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
      void closeAddApps();
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
<svelte:window onkeydown={handleDialogKeydown} />

<main class="settings-shell" data-visual-theme="familiar-workspace" data-layout="list-detail-settings" aria-busy={loading}>
  <header class="settings-header">
    <div><h1>Settings</h1><p>Choose the apps that open with your workspace.</p></div>
    <button class="secondary-button" type="button" onclick={exportDiagnostics}>
      <span aria-hidden="true"><IconDownloadSimpleRegular /></span>
      Export diagnostics
    </button>
  </header>

  <div class="settings-messages">
    {#if actionError}<p class="settings-message settings-message--error" role="alert">{actionError}</p>{/if}
    {#if notice}<p class="settings-message" role="status">{notice}</p>{/if}
    {#if validationErrors.length > 0}
      <div class="settings-message settings-message--error" role="alert">
        {#each validationErrors as error}<div>{error}</div>{/each}
      </div>
    {/if}
  </div>

  <section class="settings-content" aria-label="Workflow settings">
    <aside class="workflow-pane" aria-label="Workflow">
      <header class="pane-header">
        <div><h2>Workflow</h2><span>{apps.length}</span></div>
        <button class="primary-button primary-button--compact" type="button" bind:this={addAppButton} onclick={openAddApps}>
          <span aria-hidden="true"><IconPlusRegular /></span>
          Add app
        </button>
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
              <div class="settings-target__actions" aria-label={`Reorder ${app.name}`}>
                <button type="button" aria-label={`Move ${app.name} up`} disabled={index === 0} onclick={() => { selectedId = app.id; moveSelected(-1); }}><IconArrowUpRegular /></button>
                <button type="button" aria-label={`Move ${app.name} down`} disabled={index === apps.length - 1} onclick={() => { selectedId = app.id; moveSelected(1); }}><IconArrowDownRegular /></button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </aside>

    <section class="detail-pane" aria-label="App details">
      {#if selected}
        <header class="detail-identity">
          {#if selected.iconDataUrl}<img src={selected.iconDataUrl} alt="" />{:else}<span class="settings-icon settings-icon--large" aria-hidden="true"><IconAppWindowRegular /></span>{/if}
          <div><h2>{selected.name}</h2><p>{selected.enabled ? 'Opens with your workspace' : 'Disabled'}</p></div>
          <label class="switch-control">
            <span class="sr-only">Enable this app</span>
            <input type="checkbox" checked={selected.enabled} onchange={(event) => updateSelected({ enabled: event.currentTarget.checked })} />
            <span aria-hidden="true"></span>
          </label>
        </header>

        <section class="setting-group">
          <h3>General</h3>
          <label class="setting-row"><span>Name</span><input value={selected.name} maxlength="128" oninput={(event) => updateSelected({ name: event.currentTarget.value })} /></label>
        </section>

        <section class="setting-group">
          <h3>Launch</h3>
          <div class="setting-row"><span>Launch target</span><code title={selected.launchRef.value}>{selected.launchRef.value}</code></div>
          {#if selected.validationError}
            <div class="repair-callout" role="alert"><span>{selected.validationError}</span><button class="secondary-button" type="button" aria-label={`Choose executable for ${selected.name}`} onclick={repairSelected}>Choose executable</button></div>
          {/if}
        </section>

        <section class="setting-group settings-arguments">
          <h3>Arguments</h3>
          <p>Each value is passed separately. Shell syntax is never interpreted.</p>
          {#each selected.arguments as argument, index}
            <div class="settings-argument"><label for={`argument-${selected.id}-${index}`} class="sr-only">Argument {index + 1}</label><input id={`argument-${selected.id}-${index}`} value={argument} oninput={(event) => updateArgument(index, event.currentTarget.value)} /><button class="icon-button icon-button--small" type="button" aria-label={`Remove argument ${index + 1}`} onclick={() => removeArgument(index)}><IconXRegular /></button></div>
          {/each}
          <button class="secondary-button secondary-button--compact" type="button" onclick={addArgument}><span aria-hidden="true"><IconPlusRegular /></span>Add argument</button>
        </section>

        <button class="danger-row" type="button" aria-label={`Remove ${selected.name}`} onclick={removeSelected}>Remove app</button>
      {:else}
        <p class="settings-empty">Select an app to edit its settings.</p>
      {/if}
    </section>

  </section>

  <footer class="settings-footer"><span aria-live="polite">{saving ? 'Saving settings…' : ''}</span><button class="primary-button" type="button" disabled={!canSave} onclick={save}>{saving ? 'Saving…' : 'Save changes'}</button></footer>

  {#if showAddApps}
    <div class="dialog-backdrop">
      <div class="add-apps-dialog" role="dialog" aria-modal="true" aria-labelledby="add-apps-title" bind:this={addAppsDialog}>
        <header class="dialog-header">
          <div>
            <h2 id="add-apps-title">Add apps</h2>
            <p>Choose an installed app or browse for an executable.</p>
          </div>
          <button class="icon-button" type="button" aria-label="Close Add apps" onclick={closeAddApps}>
            <span aria-hidden="true"><IconXRegular /></span>
          </button>
        </header>
        <label class="search-field" for="discovery-search">
          <span class="sr-only">Search installed apps</span>
          <span aria-hidden="true"><IconMagnifyingGlassRegular /></span>
          <input id="discovery-search" placeholder="Search installed apps" bind:value={search} />
        </label>
        {#if discoveryFailed}
          <p class="settings-empty" role="status">Could not load installed apps.</p>
        {:else if shownDiscovered.length === 0}
          <p class="settings-empty">No installed apps match.</p>
        {:else}
          <ul class="settings-discovery-list" aria-label="Installed apps">
            {#each shownDiscovered as app (app.id)}
              <li>
                {#if app.iconDataUrl}
                  <img src={app.iconDataUrl} alt="" />
                {:else}
                  <span class="settings-icon" aria-hidden="true"><IconAppWindowRegular /></span>
                {/if}
                <span title={app.name}>{app.name}</span>
                <button class="quiet-button" type="button" aria-label={`Add ${app.name}`} onclick={() => addDiscovered(app)}>Add</button>
              </li>
            {/each}
          </ul>
        {/if}
        <div class="dialog-footer">
          <button class="secondary-button" type="button" onclick={addExecutable}>Add executable</button>
        </div>
      </div>
    </div>
  {/if}
</main>
