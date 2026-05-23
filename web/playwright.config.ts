import { defineConfig } from '@playwright/test';

const basePath = process.env.E2E_BASE_PATH ?? '';

export default defineConfig({
  testDir: './tests/e2e',
  use: { baseURL: 'http://127.0.0.1:4173', headless: true },
  webServer: {
    command: 'node tests/e2e/static-server.mjs',
    url: `http://127.0.0.1:4173${basePath || '/'}`,
    reuseExistingServer: !process.env.CI,
  },
});
