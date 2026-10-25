import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

import Settings from './Settings.svelte';
import type { AppTarget, WorkstartService } from './lib/workstart-service';

const target = (overrides: Partial<AppTarget> = {}): AppTarget => ({
  id: 'antigravity',
  name: 'Antigravity IDE',
  launchRef: { kind: 'executable', value: '/usr/bin/antigravity' },
  arguments: [],
  processMatchers: [],
  enabled: true,
  ...overrides,
});

function service(overrides: Record<string, unknown> = {}) {
  return {
    getWorkflow: vi.fn().mockResolvedValue({ schemaVersion: 1, apps: [target()] }),
    discoverInstalledApps: vi.fn().mockResolvedValue([
      { id: 'brave', name: 'Brave Browser', launchRef: { kind: 'executable', value: '/usr/bin/brave' }, processMatchers: [] },
    ]),
    getAppIcon: vi.fn().mockResolvedValue(undefined),
    pickExecutable: vi.fn().mockResolvedValue(undefined),
    saveWorkflow: vi.fn().mockResolvedValue(undefined),
    exportDiagnostics: vi.fn().mockResolvedValue(undefined),
    refreshMainWindow: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  } as unknown as WorkstartService;
}

afterEach(cleanup);

describe('Workstart settings', () => {
  it('filters discovered apps and adds a selected result to the workflow', async () => {
    const api = service();
    render(Settings, { props: { service: api } });

    await screen.findByText('Antigravity IDE');
    await fireEvent.input(screen.getByLabelText('Search installed apps'), { target: { value: 'brave' } });

    const discovered = await screen.findByRole('list', { name: 'Installed apps' });
    expect(within(discovered).getByText('Brave Browser')).toBeInTheDocument();
    await fireEvent.click(within(discovered).getByRole('button', { name: 'Add Brave Browser' }));

    expect(await screen.findByText('Brave Browser', { selector: '.settings-target__name' })).toBeInTheDocument();
  });

  it('edits safe individual arguments and rejects NUL values before save', async () => {
    const api = service();
    render(Settings, { props: { service: api } });

    await screen.findByText('Antigravity IDE');
    await fireEvent.click(screen.getByRole('button', { name: 'Add argument' }));
    const argument = screen.getByLabelText('Argument 1');
    await fireEvent.input(argument, { target: { value: '$(not a command)' } });
    expect(argument).toHaveValue('$(not a command)');

    await fireEvent.input(argument, { target: { value: 'bad\0argument' } });
    expect(screen.getByRole('alert')).toHaveTextContent('Arguments cannot contain NUL characters.');
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeDisabled();
    expect(api.saveWorkflow).not.toHaveBeenCalled();
  });

  it('repairs unresolved targets through the executable picker without exposing a command field', async () => {
    const api = service({
      getWorkflow: vi.fn().mockResolvedValue({ schemaVersion: 1, apps: [target({ validationError: 'Executable could not be resolved.' })] }),
      pickExecutable: vi.fn().mockResolvedValue({ path: '/opt/antigravity', name: 'Antigravity IDE', processMatchers: [] }),
    });
    render(Settings, { props: { service: api } });

    await screen.findByText('Executable could not be resolved.');
    await fireEvent.click(screen.getByRole('button', { name: 'Choose executable for Antigravity IDE' }));

    expect(api.pickExecutable).toHaveBeenCalledOnce();
    expect(screen.queryByText('Executable could not be resolved.')).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/command/i)).not.toBeInTheDocument();
  });

  it('adds a manually picked executable when discovery has no usable result', async () => {
    const api = service({
      getWorkflow: vi.fn().mockResolvedValue({ schemaVersion: 1, apps: [] }),
      discoverInstalledApps: vi.fn().mockResolvedValue([]),
      pickExecutable: vi.fn().mockResolvedValue({ path: '/opt/tools/notes', name: 'Notes', processMatchers: [] }),
    });
    render(Settings, { props: { service: api } });

    await screen.findByText('No installed apps match.');
    await fireEvent.click(screen.getByRole('button', { name: 'Add executable' }));

    expect(api.pickExecutable).toHaveBeenCalledOnce();
    expect(await screen.findByText('Notes', { selector: '.settings-target__name' })).toBeInTheDocument();
  });

  it('reorders, saves, and notifies the main window', async () => {
    const api = service({
      getWorkflow: vi.fn().mockResolvedValue({ schemaVersion: 1, apps: [target(), target({ id: 'brave', name: 'Brave' })] }),
    });
    render(Settings, { props: { service: api } });

    await screen.findByText('Brave');
    await fireEvent.click(screen.getByRole('button', { name: 'Move Brave up' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));

    await waitFor(() => expect(api.saveWorkflow).toHaveBeenCalledOnce());
    const saved = vi.mocked(api.saveWorkflow).mock.calls[0][0];
    expect(saved.apps[0].id).toBe('brave');
    expect(api.refreshMainWindow).toHaveBeenCalledOnce();
    expect(screen.getByRole('status')).toHaveTextContent('Settings saved.');
  });

  it('shows recoverable discovery and export failures', async () => {
    const api = service({
      discoverInstalledApps: vi.fn().mockRejectedValue(new Error('offline')),
      exportDiagnostics: vi.fn().mockRejectedValue(new Error('unavailable')),
    });
    render(Settings, { props: { service: api } });

    expect(await screen.findByText('Could not load installed apps.')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Export diagnostics' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Could not export diagnostics.');
  });
});
