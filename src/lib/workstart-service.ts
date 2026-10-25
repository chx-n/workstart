import { exit } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export type AppTarget = {
  id: string;
  name: string;
  launchRef: LaunchRef;
  arguments: string[];
  processMatchers: unknown[];
  enabled: boolean;
  iconDataUrl?: string;
  validationError?: string;
};

/** A backend-resolved launch reference. It is never a shell command. */
export type LaunchRef = {
  kind: string;
  value: string;
  executableHint?: string;
};

export type Workflow = {
  schemaVersion: number;
  apps: AppTarget[];
};

export type DiscoveredApp = {
  /** Stable backend discovery identifier used only for icon lookup. */
  id: string;
  name: string;
  launchRef: LaunchRef;
  processMatchers: unknown[];
  iconDataUrl?: string;
};

export type PickedExecutable = {
  path: string;
  name?: string;
  processMatchers: unknown[];
  iconDataUrl?: string;
};

export type LaunchOutcome = 'started' | 'skippedAlreadyRunning' | 'failed';

export type LaunchResult = {
  targetId: string;
  outcome: LaunchOutcome;
  errorCategory?: string;
};

export interface WorkstartService {
  getWorkflow(): Promise<Workflow>;
  launchTargets(targetIds: string[]): Promise<LaunchResult[]>;
  openSettings(): Promise<void>;
  exitApp(): Promise<void>;
  discoverInstalledApps(): Promise<DiscoveredApp[]>;
  getAppIcon(appId: string): Promise<string | undefined>;
  pickExecutable(): Promise<PickedExecutable | undefined>;
  saveWorkflow(workflow: Workflow): Promise<void>;
  exportDiagnostics(): Promise<void>;
  refreshMainWindow(): Promise<void>;
  subscribeWorkflowSaved(listener: () => void): Promise<() => void>;
}

export const tauriWorkstartService: WorkstartService = {
  getWorkflow: () => invoke<Workflow>('get_workflow'),
  launchTargets: (targetIds) => invoke<LaunchResult[]>('launch_targets', { targetIds }),
  openSettings: async () => {
    const settings = await WebviewWindow.getByLabel('settings');

    if (settings) {
      await settings.show();
      await settings.setFocus();
      return;
    }

    new WebviewWindow('settings', {
      title: 'Workstart Settings',
      url: 'index.html?window=settings',
      width: 760,
      height: 600,
      resizable: false,
      maximizable: false,
      fullscreen: false,
      center: true,
    });
  },
  exitApp: () => exit(0),
  discoverInstalledApps: () => invoke<DiscoveredApp[]>('discover_installed_apps'),
  getAppIcon: (appId) => invoke<string | undefined>('get_app_icon', { appId }),
  pickExecutable: () => invoke<PickedExecutable | undefined>('pick_executable'),
  saveWorkflow: (workflow) => invoke<void>('save_workflow', { workflow }),
  exportDiagnostics: () => invoke<void>('export_logs'),
  refreshMainWindow: async () => {
    const main = await WebviewWindow.getByLabel('main');
    await main?.emit('workflow-saved');
  },
  subscribeWorkflowSaved: (listener) => WebviewWindow.getCurrent().listen('workflow-saved', listener),
};
