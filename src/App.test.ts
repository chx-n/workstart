import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { tick } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

import App from './App.svelte';

type AppTarget = {
  id: string;
  name: string;
  launchRef: { kind: string; value: string };
  arguments: string[];
  processMatchers: unknown[];
  enabled: boolean;
  iconDataUrl?: string;
  validationError?: string;
};

type LaunchResult = {
  targetId: string;
  outcome: 'started' | 'skippedAlreadyRunning' | 'failed';
  errorCategory?: string;
};

function target(overrides: Partial<AppTarget> = {}): AppTarget {
  return {
    id: 'antigravity',
    name: 'Antigravity',
    launchRef: { kind: 'executable', value: '/usr/bin/antigravity' },
    arguments: [],
    processMatchers: [],
    enabled: true,
    ...overrides,
  };
}

function service(
  apps: AppTarget[],
  launchTargets = vi.fn<(targetIds: string[]) => Promise<LaunchResult[]>>().mockResolvedValue([]),
) {
  return {
    getWorkflow: vi.fn().mockResolvedValue({ schemaVersion: 1, apps }),
    launchTargets,
    openSettings: vi.fn().mockResolvedValue(undefined),
    exitApp: vi.fn().mockResolvedValue(undefined),
    discoverInstalledApps: vi.fn().mockResolvedValue([]),
    getAppIcon: vi.fn().mockResolvedValue(undefined),
    pickExecutable: vi.fn().mockResolvedValue(undefined),
    saveWorkflow: vi.fn().mockResolvedValue(undefined),
    exportDiagnostics: vi.fn().mockResolvedValue(undefined),
    refreshMainWindow: vi.fn().mockResolvedValue(undefined),
    subscribeWorkflowSaved: vi.fn().mockResolvedValue(() => undefined),
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolver) => {
    resolve = resolver;
  });
  return { promise, resolve };
}

async function readyStartButton() {
  const button = await screen.findByRole('button', { name: 'Start Work' });
  await waitFor(() => expect(button).toBeEnabled());
  return button;
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe('Workstart main interface', () => {
  it('shows a loading state while the workflow is being fetched', () => {
    const workflow = deferred<{ schemaVersion: number; apps: AppTarget[] }>();
    const api = service([]);
    api.getWorkflow.mockReturnValue(workflow.promise);

    render(App, { props: { service: api } });

    expect(screen.getByRole('status')).toHaveTextContent('Loading apps');
    expect(screen.getByRole('button', { name: 'Start Work' })).toBeDisabled();
  });

  it('renders enabled valid targets as ready', async () => {
    const api = service([
      target({ iconDataUrl: 'data:image/png;base64,AAAA' }),
      target({ id: 'brave', name: 'Brave', launchRef: { kind: 'desktopEntry', value: 'brave.desktop' } }),
    ]);

    render(App, { props: { service: api } });

    expect(await screen.findByRole('heading', { name: 'Workstart' })).toBeInTheDocument();
    const rows = await screen.findAllByRole('listitem');
    expect(rows).toHaveLength(2);
    expect(within(rows[0]).getByText('Ready')).toBeInTheDocument();
    expect(within(rows[1]).getByText('Ready')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Start Work' })).toBeEnabled();
    expect(screen.getByRole('img', { name: 'Antigravity' })).toHaveAttribute('src', 'data:image/png;base64,AAAA');
  });

  it('shows per-item launching and final started states', async () => {
    const pending = deferred<LaunchResult[]>();
    const launchTargets = vi.fn().mockReturnValue(pending.promise);
    const api = service([target(), target({ id: 'brave', name: 'Brave' })], launchTargets);
    render(App, { props: { service: api } });
    const button = await readyStartButton();

    await fireEvent.click(button);

    expect(launchTargets).toHaveBeenCalledWith(['antigravity', 'brave']);
    expect(screen.getAllByText('Launching')).toHaveLength(2);
    expect(button).toBeDisabled();

    pending.resolve([
      { targetId: 'antigravity', outcome: 'started' },
      { targetId: 'brave', outcome: 'started' },
    ]);

    await waitFor(() => expect(screen.getAllByText('Started')).toHaveLength(2));
    expect(screen.getByRole('status')).toHaveTextContent('Workspace ready');
  });

  it('treats skipped targets as successful and labels them already open', async () => {
    const launchTargets = vi.fn().mockResolvedValue([
      { targetId: 'antigravity', outcome: 'skippedAlreadyRunning' as const },
    ]);
    const api = service([target()], launchTargets);
    render(App, { props: { service: api } });

    await fireEvent.click(await readyStartButton());

    expect(await screen.findByText('Already open')).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Workspace ready');
  });

  it('keeps partial failures visible and retries only failed target IDs', async () => {
    const launchTargets = vi
      .fn()
      .mockResolvedValueOnce([
        { targetId: 'antigravity', outcome: 'started' as const },
        { targetId: 'brave', outcome: 'failed' as const, errorCategory: 'notFound' },
      ])
      .mockResolvedValueOnce([{ targetId: 'brave', outcome: 'started' as const }]);
    const api = service([target(), target({ id: 'brave', name: 'Brave' })], launchTargets);
    render(App, { props: { service: api } });

    await fireEvent.click(await readyStartButton());

    const retry = await screen.findByRole('button', { name: 'Try Again' });
    expect(screen.getByText('Failed')).toBeInTheDocument();
    expect(screen.getByText('Started')).toBeInTheDocument();
    expect(screen.getByText('Could not find this app.')).toBeInTheDocument();

    await fireEvent.click(retry);

    expect(launchTargets).toHaveBeenNthCalledWith(2, ['brave']);
    await waitFor(() => expect(screen.getAllByText('Started')).toHaveLength(2));
  });

  it.each([
    { label: 'empty workflow', apps: [] },
    {
      label: 'only disabled and invalid targets',
      apps: [
        target({ enabled: false }),
        target({ id: 'broken', validationError: 'Executable could not be resolved.' }),
      ],
    },
  ])('disables launch for $label', async ({ apps }) => {
    const api = service(apps);
    render(App, { props: { service: api } });

    const button = await screen.findByRole('button', { name: 'Start Work' });
    expect(button).toBeDisabled();
    expect(api.launchTargets).not.toHaveBeenCalled();
  });

  it('turns a rejected launch request into retryable item failures', async () => {
    const launchTargets = vi.fn().mockRejectedValue(new Error('IPC unavailable'));
    render(App, { props: { service: service([target()], launchTargets) } });

    await fireEvent.click(await readyStartButton());

    expect(await screen.findByText('Failed')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Try Again' })).toBeEnabled();
  });

  it('exits through the service one second after complete success', async () => {
    vi.useFakeTimers();
    const launchTargets = vi.fn().mockResolvedValue([{ targetId: 'antigravity', outcome: 'started' as const }]);
    const api = service([target()], launchTargets);
    render(App, { props: { service: api } });
    await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Start Work' })).toBeEnabled());

    await fireEvent.click(screen.getByRole('button', { name: 'Start Work' }));
    await Promise.resolve();
    await tick();
    expect(screen.getByRole('status')).toHaveTextContent('Workspace ready');

    await vi.advanceTimersByTimeAsync(999);
    expect(api.exitApp).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(api.exitApp).toHaveBeenCalledOnce();
  });

  it('opens settings from an accessible icon button', async () => {
    const api = service([target()]);
    render(App, { props: { service: api } });
    const settings = screen.getByRole('button', { name: 'Open settings' });

    expect(settings).toHaveAttribute('type', 'button');
    await fireEvent.click(settings);

    expect(api.openSettings).toHaveBeenCalledOnce();
  });

  it('shows a recoverable load error without enabling launch', async () => {
    const api = service([]);
    api.getWorkflow.mockRejectedValue(new Error('backend unavailable'));
    render(App, { props: { service: api } });

    expect(await screen.findByRole('alert')).toHaveTextContent('Could not load your workflow');
    expect(screen.getByRole('button', { name: 'Start Work' })).toBeDisabled();
  });
});
