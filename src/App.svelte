<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

  import {
    tauriWorkstartService,
    type AppTarget,
    type LaunchResult,
    type WorkstartService,
  } from './lib/workstart-service';

  type TargetStatus = 'Ready' | 'Launching' | 'Already open' | 'Started' | 'Failed' | 'Disabled';

  let { service = tauriWorkstartService }: { service?: WorkstartService } = $props();

  let apps = $state<AppTarget[]>([]);
  let statuses = $state<Record<string, TargetStatus>>({});
  let itemErrors = $state<Record<string, string>>({});
  let loading = $state(true);
  let launching = $state(false);
  let complete = $state(false);
  let loadFailed = $state(false);
  let settingsError = $state(false);
  let exitTimer: ReturnType<typeof setTimeout> | undefined;

  const validEnabledApps = $derived(apps.filter((app) => app.enabled && !app.validationError));
  const failedIds = $derived(validEnabledApps.filter((app) => statuses[app.id] === 'Failed').map((app) => app.id));
  const buttonLabel = $derived(failedIds.length > 0 ? 'Try Again' : 'Start Work');
  const canLaunch = $derived(
    !loading && !launching && !complete && (failedIds.length > 0 || validEnabledApps.length > 0),
  );
  const summary = $derived.by(() => {
    if (loading) return 'Loading apps…';
    if (loadFailed) return 'Workflow unavailable';
    if (complete) return 'Workspace ready. Closing…';
    if (launching) return 'Launching apps…';
    if (failedIds.length > 0) return 'Some apps could not be started.';
    if (validEnabledApps.length === 0) return 'No apps are ready to launch.';
    return `${validEnabledApps.length} ${validEnabledApps.length === 1 ? 'app' : 'apps'} ready`;
  });

  onMount(() => {
    void loadWorkflow();
  });

  onDestroy(() => {
    if (exitTimer !== undefined) clearTimeout(exitTimer);
  });

  async function loadWorkflow() {
    try {
      const workflow = await service.getWorkflow();
      apps = workflow.apps;
      statuses = Object.fromEntries(
        apps.map((app) => [app.id, app.validationError ? 'Failed' : app.enabled ? 'Ready' : 'Disabled']),
      );
      itemErrors = Object.fromEntries(
        apps.filter((app) => app.validationError).map((app) => [app.id, app.validationError as string]),
      );
    } catch {
      loadFailed = true;
    } finally {
      loading = false;
    }
  }

  function friendlyError(category?: string) {
    switch (category) {
      case 'notFound':
      case 'not_found':
        return 'Could not find this app.';
      case 'permissionDenied':
      case 'permission_denied':
        return 'Permission was denied.';
      case 'invalidConfiguration':
      case 'invalid_configuration':
        return 'This app is not configured correctly.';
      default:
        return 'Could not start this app.';
    }
  }

  function applyResults(targetIds: string[], results: LaunchResult[]) {
    const resultsById = new Map(results.map((result) => [result.targetId, result]));
    const nextStatuses = { ...statuses };
    const nextErrors = { ...itemErrors };

    for (const targetId of targetIds) {
      const result = resultsById.get(targetId);

      if (result?.outcome === 'started') {
        nextStatuses[targetId] = 'Started';
        delete nextErrors[targetId];
      } else if (result?.outcome === 'skippedAlreadyRunning') {
        nextStatuses[targetId] = 'Already open';
        delete nextErrors[targetId];
      } else {
        nextStatuses[targetId] = 'Failed';
        nextErrors[targetId] = friendlyError(result?.errorCategory);
      }
    }

    statuses = nextStatuses;
    itemErrors = nextErrors;
  }

  function finishIfSuccessful() {
    const allSucceeded = validEnabledApps.every((app) => {
      const status = statuses[app.id];
      return status === 'Started' || status === 'Already open';
    });

    if (!allSucceeded || validEnabledApps.length === 0) return;

    complete = true;
    exitTimer = setTimeout(() => {
      void service.exitApp();
    }, 1_000);
  }

  async function startWork() {
    if (!canLaunch) return;

    const targetIds = failedIds.length > 0 ? failedIds : validEnabledApps.map((app) => app.id);
    const nextStatuses = { ...statuses };
    const nextErrors = { ...itemErrors };

    for (const targetId of targetIds) {
      nextStatuses[targetId] = 'Launching';
      delete nextErrors[targetId];
    }

    statuses = nextStatuses;
    itemErrors = nextErrors;
    launching = true;

    try {
      applyResults(targetIds, await service.launchTargets(targetIds));
    } catch {
      applyResults(
        targetIds,
        targetIds.map((targetId) => ({ targetId, outcome: 'failed', errorCategory: 'ipc' })),
      );
    } finally {
      launching = false;
    }

    finishIfSuccessful();
  }

  async function openSettings() {
    settingsError = false;
    try {
      await service.openSettings();
    } catch {
      settingsError = true;
    }
  }
</script>

<svelte:head>
  <title>Workstart</title>
</svelte:head>

<main class="app-shell">
  <header class="app-header">
    <div>
      <h1>Workstart</h1>
      <p>Start your workspace in one click.</p>
    </div>
    <button class="icon-button" type="button" aria-label="Open settings" onclick={openSettings}>
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 15.25A3.25 3.25 0 1 0 12 8.75a3.25 3.25 0 0 0 0 6.5Zm7.05-2.06a7.84 7.84 0 0 0 .04-2.12l1.66-1.3-1.8-3.12-1.98.79a8.2 8.2 0 0 0-1.83-1.07L14.85 4h-3.6l-.3 2.37a8.2 8.2 0 0 0-1.82 1.07l-1.98-.79-1.8 3.12 1.66 1.3a7.84 7.84 0 0 0 .04 2.12l-1.7 1.34 1.8 3.12 2.02-.81c.55.44 1.16.8 1.81 1.05l.27 2.11h3.6l.27-2.11a8.2 8.2 0 0 0 1.81-1.05l2.02.81 1.8-3.12-1.7-1.34Z"
        />
      </svg>
    </button>
  </header>

  <section class="target-panel" aria-label="Apps in your workflow">
    {#if loading}
      <div class="state-card" aria-hidden="true">
        <span class="spinner"></span>
        <span>Loading your workflow</span>
      </div>
    {:else if loadFailed}
      <div class="state-card state-card--error" role="alert">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 2.5 20h19L12 3Zm0 5.5v6m0 2.5v.5" /></svg>
        <span><strong>Could not load your workflow</strong><small>Open Workstart again to retry.</small></span>
      </div>
    {:else if apps.length === 0}
      <div class="state-card">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 6.5h14v11H5zM8 10h8m-8 3h5" /></svg>
        <span><strong>No apps yet</strong><small>Add apps from Settings to get started.</small></span>
      </div>
    {:else}
      <ul class="target-list">
        {#each apps as app (app.id)}
          <li class:target-row--muted={!app.enabled} class:target-row--failed={statuses[app.id] === 'Failed'}>
            <span class="app-icon">
              {#if app.iconDataUrl}
                <img src={app.iconDataUrl} alt={app.name} />
              {:else}
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M5.5 5.5h5v5h-5zm8 0h5v5h-5zm-8 8h5v5h-5zm8 0h5v5h-5z" />
                </svg>
              {/if}
            </span>
            <span class="target-copy">
              <strong title={app.name}>{app.name}</strong>
              {#if itemErrors[app.id]}
                <small>{itemErrors[app.id]}</small>
              {/if}
            </span>
            <span class="target-status" data-status={statuses[app.id]}>
              {#if statuses[app.id] === 'Launching'}<span class="status-dot status-dot--pulse"></span>{/if}
              {statuses[app.id]}
            </span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <footer class="action-area">
    {#if settingsError}<p class="settings-error" role="alert">Could not open Settings.</p>{/if}
    <p class="summary" role="status" aria-live="polite">{summary}</p>
    <button class="primary-button" type="button" disabled={!canLaunch} onclick={startWork}>
      {#if launching}<span class="button-spinner" aria-hidden="true"></span>{/if}
      {complete ? 'Work started' : buttonLabel}
    </button>
  </footer>
</main>
