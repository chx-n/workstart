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

import { tauriWorkstartService, type Workflow } from './workstart-service';

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

  it('uses only fixed settings IPC commands and payloads', async () => {
    const workflow: Workflow = { schemaVersion: 1, apps: [] };
    const discovered = [{ id: 'brave', name: 'Brave', launchRef: { kind: 'executable', value: '/usr/bin/brave' }, processMatchers: [] }];
    vi.mocked(invoke)
      .mockResolvedValueOnce(discovered)
      .mockResolvedValueOnce('data:image/png;base64,AAAA')
      .mockResolvedValueOnce({ path: '/usr/bin/brave', name: 'Brave', processMatchers: [] })
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined);

    await expect(tauriWorkstartService.discoverInstalledApps()).resolves.toEqual(discovered);
    await expect(tauriWorkstartService.getAppIcon('brave')).resolves.toBe('data:image/png;base64,AAAA');
    await expect(tauriWorkstartService.pickExecutable()).resolves.toEqual({
      path: '/usr/bin/brave',
      name: 'Brave',
      processMatchers: [],
    });
    await tauriWorkstartService.saveWorkflow(workflow);
    await tauriWorkstartService.exportDiagnostics();

    expect(invoke).toHaveBeenNthCalledWith(1, 'discover_installed_apps');
    expect(invoke).toHaveBeenNthCalledWith(2, 'get_app_icon', { appId: 'brave' });
    expect(invoke).toHaveBeenNthCalledWith(3, 'pick_executable');
    expect(invoke).toHaveBeenNthCalledWith(4, 'save_workflow', { workflow });
    expect(invoke).toHaveBeenNthCalledWith(5, 'export_logs');
  });
});
