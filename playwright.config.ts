import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  use: {
    browserName: 'chromium',
    baseURL: 'http://127.0.0.1:4173',
    channel: 'chrome',
    trace: 'on-first-retry'
  },
  webServer: {
    command: 'python3 -m http.server 4173 --bind 127.0.0.1 --directory dist',
    url: 'http://127.0.0.1:4173/',
    reuseExistingServer: true
  }
});
