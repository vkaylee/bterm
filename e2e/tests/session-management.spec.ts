import { test, expect } from '@playwright/test';

test.describe('Session Management', () => {
  const SESSION_NAME = `mgmt-session-${Date.now()}-${Math.floor(Math.random() * 1000)}`;

  test('should create and delete a session', async ({ page }) => {
    await page.goto('/');

    // Create session
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

    // Wait for the session list to update and specifically locate the card
    const sessionCardLocator = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    await expect(sessionCardLocator).toBeVisible();
    await expect(sessionCardLocator).toHaveCount(1); // Ensure it's unique after creation

    // Confirm deletion in the alert dialog
    page.on('dialog', async dialog => {
      expect(dialog.message()).toContain(`Are you sure you want to delete session "${SESSION_NAME}"?`);
      await dialog.accept();
    });

    // Delete session
    const deleteButton = sessionCardLocator.locator('button', { has: page.locator('svg') });
    await deleteButton.click();

    // Give frontend some time to update
    await page.waitForTimeout(500); // Wait for 500ms

    // Wait for the session card to be removed from the list
    await expect(sessionCardLocator).not.toBeVisible();
    await expect(sessionCardLocator).toHaveCount(0);
  });
});
