import { test, expect } from '../fixtures';

test.describe('Terminal Interaction', () => {
  const SESSION_NAME = `term-session-${Date.now()}-${Math.floor(Math.random() * 1000)}`;

  test.beforeEach(async ({ page }) => {
    // Default dialog handler
    page.on('dialog', async dialog => {
        try { await dialog.accept(); } catch(e) {}
    });

    // Ensure we start from a clean state or create a session
    await page.goto('/');
    await page.fill('#new-session-name', SESSION_NAME);
    const [response] = await Promise.all([
      page.waitForResponse(response => response.url().includes('/api/sessions') && response.request().method() === 'POST'),
      page.click('button:has-text("Create Session")')
    ]);
    expect(response.ok()).toBeTruthy(); // Ensure API call was successful
    
    await expect(page.locator('#terminal-view')).toBeVisible(); // Wait for terminal view to be visible
  });

  test.afterEach(async ({ page }) => {
    // Clean up: go back to dashboard if not already there, and delete the session
    const isDashboardVisible = await page.locator('#dashboard').isVisible();
    if (!isDashboardVisible) {
        await page.click('button[title="Exit Session"]'); // Back to dashboard
        await expect(page.locator('#dashboard')).toBeVisible();
    }
    
    const sessionCardToDelete = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    if (await sessionCardToDelete.isVisible()) {
        const deleteButton = sessionCardToDelete.locator('button', { has: page.locator('svg') });
        await deleteButton.click();
        await expect(sessionCardToDelete).toBeHidden({ timeout: 5000 });
    }
  });

  test('should send command and receive output', async ({ page }, testInfo) => {
    page.on('console', msg => {
        console.log(`[BROWSER] ${msg.type()}: ${msg.text()}`);
    });

    const command = 'echo TEST_OUTPUT_SIGNAL'; 
    const expectedOutput = 'TEST_OUTPUT_SIGNAL';

    // Ensure terminal view is visible and focusable
    await expect(page.locator('#terminal-view')).toBeVisible();
    
    // Wait for WebSocket to be open
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);

    // Focus terminal and monkey-patch write
    await page.evaluate(() => {
        window.term.focus();
        (window as any).terminalOutput = '';
        const originalWrite = window.term.write.bind(window.term);
        window.term.write = (data: any) => {
            let decoded = '';
            if (typeof data === 'string') {
                decoded = data;
            } else {
                decoded = new TextDecoder().decode(data);
            }
            (window as any).terminalOutput += decoded;
            originalWrite(data);
        };
    });
    await page.waitForTimeout(1000); 
    
    // Send input directly via WebSocket
    await page.evaluate(({ cmd }) => {
        (window as any).ws.send(JSON.stringify({ type: 'Input', data: cmd + '\n' }));
    }, { cmd: command });

    // Wait for the output to appear in the captured output
    try {
        await page.waitForFunction(
          ([output]) => {
            return (window as any).terminalOutput && (window as any).terminalOutput.includes(output);
          },
          [expectedOutput],
          { timeout: 15000 }
        );
    } catch (e) {
        const captured = await page.evaluate(() => (window as any).terminalOutput);
        console.log(`Captured output for ${testInfo.title} (${testInfo.project.name}):\n${captured}`);
        throw e;
    }

    // Final verification
    const finalCaptured = await page.evaluate(() => (window as any).terminalOutput);
    expect(finalCaptured).toContain(expectedOutput);
  });
    
  test('should automatically exit terminal on shell exit', async ({ page }) => {
    // Ensure WebSocket is ready
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);

    // Focus terminal
    await page.click('#terminal');
    
    // Type 'exit' and Enter
    await page.keyboard.type('exit');
    await page.keyboard.press('Enter');

    // Wait for dashboard to become visible
    await expect(page.locator('#dashboard')).toBeVisible({ timeout: 10000 });
    
    // Check if session-info is hidden
    await expect(page.locator('#session-info')).toBeHidden();

    // Ensure the session is actually removed from the dashboard list
    const sessionCard = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    await expect(sessionCard).not.toBeVisible();
  });
});
