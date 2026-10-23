// @vitest-environment node

import { describe, expect, it } from 'vitest';

import config from '../vite.config';

describe('Vite configuration', () => {
  it('serves the development frontend on the URL expected by Tauri', () => {
    expect(config.server).toMatchObject({
      port: 1420,
      strictPort: true,
    });
  });
});
