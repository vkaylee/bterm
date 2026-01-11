import { test, expect } from '../fixtures';

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
    
    await page.waitForSelector('#terminal'); // Wait for terminal to be visible
  });

  test.afterEach(async ({ page }) => {
    // Clean up: go back to dashboard if not already there, and delete the session
    const isDashboardVisible = await page.locator('#dashboard').isVisible();
    if (!isDashboardVisible) {
        await page.click('button[title="Exit Session"]'); // Back to dashboard
        await page.waitForSelector('#dashboard');
    }
    
    const sessionCardToDelete = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    const exists = await sessionCardToDelete.isVisible();
    
    if (exists) {
        // Set up dialog handler before clicking delete
        page.on('dialog', async dialog => {
          await dialog.accept();
        });

        const deleteButton = sessionCardToDelete.locator('button', { has: page.locator('svg') });
        await deleteButton.click();

        // Give frontend some time to update
        await page.waitForTimeout(500); // Wait for 500ms

        await expect(sessionCardToDelete).not.toBeVisible();
    }
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
    
      test('should automatically exit terminal on shell exit', async ({ page }) => {
        // Ensure WebSocket is ready
        await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);

        // Focus terminal
        await page.click('#terminal');
        
        // Type 'exit' and Enter
        await page.keyboard.type('exit');
        await page.keyboard.press('Enter');
    
            // Wait for dashboard to become visible (proving the auto-exit worked)
            // We increase timeout as 'exit' might take a moment to propagate
            await expect(page.locator('#dashboard')).toBeVisible({ timeout: 10000 });
            
            // Check if session-info is hidden
            await expect(page.locator('#session-info')).toBeHidden();
        
            // NEW CHECK: Ensure the session is actually removed from the dashboard list
            const sessionCard = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
            await expect(sessionCard).not.toBeVisible();
          });
        });
        