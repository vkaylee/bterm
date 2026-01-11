import { test, expect } from '../fixtures';

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

    // Verify auto-join: Terminal should be visible and ID should match
    await expect(page.locator('#terminal-view')).toBeVisible();
    await expect(page.locator('#current-session-id')).toHaveText(SESSION_NAME);

    // Go back to dashboard to verify deletion flow
    await page.click('button[title="Exit Session"]');
    await expect(page.locator('#dashboard')).toBeVisible();

    // Wait for the new session card to appear in the DOM
    const sessionCardLocator = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    await expect(sessionCardLocator).toBeVisible();
    await expect(sessionCardLocator).toHaveCount(1); // Ensure it's unique after creation

    // Confirm deletion in the alert dialog AND handle the SSE notification alert
    page.on('dialog', async dialog => {
      if (dialog.message().includes('Are you sure')) {
        await dialog.accept();
      } else if (dialog.message().includes('has been deleted')) {
        await dialog.accept();
      }
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

  test('should create a session by pressing Enter key', async ({ page }) => {
    const ENTER_SESSION = `enter-session-${Date.now()}`;
    await page.goto('/');

    await page.fill('#new-session-name', ENTER_SESSION);
    
    // Press Enter and wait for the terminal view to appear
    await page.press('#new-session-name', 'Enter');

    // Verify auto-join
    await expect(page.locator('#terminal-view')).toBeVisible();
    await expect(page.locator('#current-session-id')).toHaveText(ENTER_SESSION);
  });

  test('should hide active sessions section when no sessions exist', async ({ page }) => {
    // Navigate and wait for the sessions API response
    const sessionsResponse = page.waitForResponse(resp => resp.url().includes('/api/sessions') && resp.request().method() === 'GET');
    await page.goto('/');
    await sessionsResponse;
    
    // Initially, there should be no sessions, so section should be hidden
    await expect(page.locator('#active-sessions-section')).toBeHidden({ timeout: 10000 });
    
    // Verify auto-focus
    await expect(page.locator('#new-session-name')).toBeFocused();
    
    // Create one
    await page.fill('#new-session-name', 'temp-session');
    await page.click('button:has-text("Create Session")');
    
    // Go back to dashboard
    await page.click('button[title="Exit Session"]');
    await expect(page.locator('#active-sessions-section')).toBeVisible();
    
    // Delete it
    page.on('dialog', dialog => dialog.accept());
    const deleteButton = page.locator('#session-list button').first();
    await deleteButton.click();
    
    // Section should be hidden again
    await expect(page.locator('#active-sessions-section')).toBeHidden();
  });
});
