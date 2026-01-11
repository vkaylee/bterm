import { test, expect } from '@playwright/test';

test.describe('Terminal Interaction', () => {
  const SESSION_NAME = `term-session-${Date.now()}-${Math.floor(Math.random() * 1000)}`;

  test.beforeEach(async ({ page }) => {
    // Ensure we start from a clean state or create a session
    await page.goto('/');
    await page.fill('#new-session-name', SESSION_NAME);
    const [response] = await Promise.all([
      page.waitForResponse(response => response.url().includes('/api/sessions') && response.request().method() === 'POST'),
      page.click('button:has-text("Create Session")')
    ]);
    expect(response.ok()).toBeTruthy(); // Ensure API call was successful
    
    // Wait for the new session card to appear in the DOM
    await page.waitForFunction(
      (sessionName) => {
        const list = document.getElementById('session-list');
        // Check innerHTML for presence of session name
        return list && list.innerHTML.includes(sessionName);
      },
      SESSION_NAME,
      { timeout: 30000 } // Increased timeout
    );
    
    // Wait for the session card to appear and then click it to join
    const sessionCardToJoin = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    await expect(sessionCardToJoin).toBeVisible();
    await sessionCardToJoin.click();
    
    await page.waitForSelector('#terminal'); // Wait for terminal to be visible
  });

  test.afterEach(async ({ page }) => {
    // Clean up: go back to dashboard and delete the session
    await page.click('button[title="Exit Session"]'); // Back to dashboard
    await page.waitForSelector('#dashboard');
    
    const sessionCardToDelete = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    await expect(sessionCardToDelete).toBeVisible(); // Ensure it's visible before trying to delete
    
    // Set up dialog handler before clicking delete
    page.on('dialog', async dialog => {
      await dialog.accept();
    });

    const deleteButton = sessionCardToDelete.locator('button', { has: page.locator('svg') });
    await deleteButton.click();

    // Give frontend some time to update
    await page.waitForTimeout(500); // Wait for 500ms

    await expect(sessionCardToDelete).not.toBeVisible();
  });

  test('should send command and receive output', async ({ page }) => {
    const command = 'echo Hello Playwright!';
    const expectedOutput = 'Hello Playwright!';

    // Focus terminal first
    await page.click('#terminal');
    
    // Type command using native keyboard events
    await page.keyboard.type(command);
    await page.keyboard.press('Enter');

    // Wait for the output to appear
    await page.waitForFunction(
      ([output]) => {
        const terminalElement = document.querySelector('#terminal .xterm-rows');
        const textContent = terminalElement?.textContent;
        // xterm.js might add control characters, so just check for inclusion
        return textContent && textContent.includes(output);
      },
      [expectedOutput],
      { timeout: 30000 }
    );

    // Verify the output
    const terminalContent = await page.$eval('#terminal .xterm-rows', el => el.textContent);
    expect(terminalContent).toContain(expectedOutput);
  });
});