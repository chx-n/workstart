import { exit } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export type AppTarget = {
  id: string;
  name: string;
  launchRef: unknown;
  arguments: string[];
  processMatchers: unknown[];
  enabled: boolean;
  iconDataUrl?: string;
  validationError?: string;
};

export type Workflow = {
  schemaVersion: number;
  apps: AppTarget[];
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
};
