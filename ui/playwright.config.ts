import { defineConfig, devices } from '@playwright/test';

// WebKit is the closest cross-platform proxy for WKWebView. The UI runs against the mocked
// command layer (`vite --mode mock`); no Tauri runtime is involved.
//
// Escape hatch for machines that cannot download WebKit (for example behind an allow-listing
// proxy): `PW_CHANNEL=chrome make e2e` runs the same specs in an installed Chrome. CI always
// uses WebKit.
const channel = process.env['PW_CHANNEL'];
const browser = channel
  ? { ...devices['Desktop Chrome'], channel }
  : { ...devices['Desktop Safari'] };

export default defineConfig({
  testDir: './e2e',
  retries: process.env['CI'] ? 1 : 0,
  reporter: process.env['CI'] ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: 'http://127.0.0.1:1420',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'desktop',
      use: { ...browser, viewport: { width: 1280, height: 800 } },
    },
    {
      name: 'ipad',
      use: { ...browser, viewport: { width: 1180, height: 820 }, hasTouch: true },
    },
    {
      name: 'ipad-portrait',
      use: { ...browser, viewport: { width: 820, height: 1180 }, hasTouch: true },
    },
    {
      // Split View at one third of an iPad, or a narrow Mac window (E-FB-9).
      name: 'narrow',
      use: { ...browser, viewport: { width: 420, height: 800 }, hasTouch: true },
    },
  ],
  webServer: {
    command: 'npx vite --mode mock',
    url: 'http://127.0.0.1:1420',
    reuseExistingServer: !process.env['CI'],
    timeout: 60_000,
  },
});
