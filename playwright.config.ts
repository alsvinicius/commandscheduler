import { defineConfig } from '@playwright/test'

const browserPath = process.env.CHROME_BIN

export default defineConfig({
  testDir: './tests/web',
  fullyParallel: false,
  workers: 1,
  timeout: 30_000,
  reporter: 'list',
  use: {
    browserName: 'chromium',
    headless: true,
    launchOptions: browserPath ? { executablePath: browserPath } : {},
  },
})
