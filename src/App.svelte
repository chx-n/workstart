<script lang="ts">
  import IconAppWindowRegular from 'phosphor-icons-svelte/IconAppWindowRegular.svelte';
  import IconCheckCircleFill from 'phosphor-icons-svelte/IconCheckCircleFill.svelte';
  import IconGearRegular from 'phosphor-icons-svelte/IconGearRegular.svelte';
  import IconProhibitRegular from 'phosphor-icons-svelte/IconProhibitRegular.svelte';
  import IconSpinnerGapRegular from 'phosphor-icons-svelte/IconSpinnerGapRegular.svelte';
  import IconWarningCircleRegular from 'phosphor-icons-svelte/IconWarningCircleRegular.svelte';
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
  let unlistenWorkflowSaved: (() => void) | undefined;

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
    void service.subscribeWorkflowSaved(() => {
      loading = true;
      loadFailed = false;
      complete = false;
      void loadWorkflow();
    }).then((unlisten) => {
      unlistenWorkflowSaved = unlisten;
    });
  });

  onDestroy(() => {
    if (exitTimer !== undefined) clearTimeout(exitTimer);
    unlistenWorkflowSaved?.();
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

<main class="app-shell" data-visual-theme="familiar-workspace" data-layout="launcher">
  <header class="app-header">
    <div class="app-heading">
      <h1>Workstart</h1>
      <p>Start your workspace in one click.</p>
    </div>
    <button class="icon-button" type="button" aria-label="Open settings" title="Settings" onclick={openSettings}>
      <span aria-hidden="true"><IconGearRegular /></span>
    </button>
  </header>

  <section class="workflow-surface" aria-label="Apps in your workflow">
    {#if loading}
      <div class="state-panel" aria-hidden="true">
        <span class="spinner"><IconSpinnerGapRegular /></span>
        <span>Loading your workflow</span>
      </div>
    {:else if loadFailed}
      <div class="state-panel state-panel--error" role="alert">
        <span aria-hidden="true"><IconWarningCircleRegular /></span>
        <span><strong>Could not load your workflow</strong><small>Open Workstart again to retry.</small></span>
      </div>
    {:else if apps.length === 0}
      <div class="state-panel">
        <span aria-hidden="true"><IconAppWindowRegular /></span>
        <span><strong>No apps yet</strong><small>Add apps from Settings to get started.</small></span>
      </div>
    {:else}
      <ul class="target-list" aria-label="Apps in your workflow">
        {#each apps as app (app.id)}
          <li class:target-row--muted={!app.enabled} class:target-row--failed={statuses[app.id] === 'Failed'}>
            <span class="app-icon">
              {#if app.iconDataUrl}
                <img src={app.iconDataUrl} alt={app.name} />
              {:else}
                <IconAppWindowRegular />
              {/if}
            </span>
            <span class="target-copy">
              <strong title={app.name}>{app.name}</strong>
              {#if itemErrors[app.id]}
                <small>{itemErrors[app.id]}</small>
              {/if}
            </span>
            <span class="target-status" data-status={statuses[app.id]}>
              <span class="status-icon" aria-label={statuses[app.id]}>
                {#if statuses[app.id] === 'Failed'}
                  <IconWarningCircleRegular />
                {:else if statuses[app.id] === 'Launching'}
                  <IconSpinnerGapRegular class="status-icon--spinning" />
                {:else if statuses[app.id] === 'Disabled'}
                  <IconProhibitRegular />
                {:else}
                  <IconCheckCircleFill />
                {/if}
              </span>
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
</main>
