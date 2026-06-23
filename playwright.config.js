// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './tests',
  timeout:       60_000,  // WASM compile can be slow on first load
  retries:       1,
  workers:       1,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],

  use: {
    baseURL: 'http://127.0.0.1:7777',
    headless: true,
    navigationTimeout: 30_000,
    actionTimeout:     30_000,
  },

  // Start the static file server before running tests
  webServer: {
    command: 'node serve.js 7777',
    url:     'http://127.0.0.1:7777',
    reuseExistingServer: !process.env.CI,
    timeout: 15_000,
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
