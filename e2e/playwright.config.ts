import { defineConfig, devices } from '@playwright/test';
import * as path from 'path';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// require('dotenv').config();

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './tests',
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['list'], // Shows results directly in the CLI
    ['html', { open: 'never' }] // Generates HTML report but does not open the server
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL is now managed via worker fixtures */
    // baseURL: 'http://localhost:3000',

    /* Collect traces upon failing the first time. */
    trace: 'on-first-retry',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'WebGL',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: process.env.BASE_URL ? `${process.env.BASE_URL}?renderer=webgl` : undefined,
      },
    },
    {
      name: 'DOM-Fallback',
      use: { 
        ...devices['Desktop Chrome'],
        // This will be appended to URLs in page.goto('/')
        baseURL: process.env.BASE_URL ? `${process.env.BASE_URL}?renderer=dom` : undefined,
      },
    },
  ],

  /* Run your local dev server before starting the tests */
  /* webServer: {
    command: 'cargo run',
    url: 'http://localhost:3000',
    timeout: 60 * 1000, // 60 seconds
    reuseExistingServer: !process.env.CI,
    cwd: path.resolve(__dirname, '..'), // Run cargo run from the project root
  }, */
});
