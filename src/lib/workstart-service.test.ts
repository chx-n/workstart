import { exit } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const windowMocks = vi.hoisted(() => ({
  construct: vi.fn(),
  getByLabel: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/app', () => ({ exit: vi.fn() }));
vi.mock('@tauri-apps/api/webviewWindow', () => {
  function WebviewWindow(this: unknown, ...args: unknown[]) {
    return windowMocks.construct(...args);
  }

  WebviewWindow.getByLabel = windowMocks.getByLabel;
  return { WebviewWindow };
});

import { tauriWorkstartService } from './workstart-service';

beforeEach(() => {
  vi.clearAllMocks();
});

describe('tauriWorkstartService', () => {
  it('loads the workflow through the fixed get_workflow command', async () => {
    const workflow = { schemaVersion: 1, apps: [] };
    vi.mocked(invoke).mockResolvedValue(workflow);

    await expect(tauriWorkstartService.getWorkflow()).resolves.toEqual(workflow);
    expect(invoke).toHaveBeenCalledWith('get_workflow');
  });

  it('sends target IDs through the fixed launch_targets command', async () => {
    const results = [{ targetId: 'brave', outcome: 'started' as const }];
    vi.mocked(invoke).mockResolvedValue(results);

    await expect(tauriWorkstartService.launchTargets(['brave'])).resolves.toEqual(results);
    expect(invoke).toHaveBeenCalledWith('launch_targets', { targetIds: ['brave'] });
  });

  it('shows and focuses the configured settings WebviewWindow', async () => {
    const settings = {
      show: vi.fn().mockResolvedValue(undefined),
      setFocus: vi.fn().mockResolvedValue(undefined),
    };
    windowMocks.getByLabel.mockResolvedValue(settings);

    await tauriWorkstartService.openSettings();

    expect(windowMocks.getByLabel).toHaveBeenCalledWith('settings');
    expect(settings.show).toHaveBeenCalledOnce();
    expect(settings.setFocus).toHaveBeenCalledOnce();
  });

  it('creates the settings window with the fixed label when it is absent', async () => {
    windowMocks.getByLabel.mockResolvedValue(null);
    windowMocks.construct.mockReturnValue({});

    await tauriWorkstartService.openSettings();

    expect(windowMocks.construct).toHaveBeenCalledWith(
      'settings',
      expect.objectContaining({ title: 'Workstart Settings' }),
    );
  });

  it('exits through the Tauri app process API', async () => {
    vi.mocked(exit).mockResolvedValue(undefined);

    await tauriWorkstartService.exitApp();

    expect(exit).toHaveBeenCalledWith(0);
  });
});
