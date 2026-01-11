import { test, expect } from '../fixtures';

test.describe('Real-time Synchronization', () => {
  test('should sync session deletion across multiple pages', async ({ context, baseURL }) => {
    const page1 = await context.newPage();
    const page2 = await context.newPage();
    const SESSION_ID = `sync-test-${Date.now()}`;

    // 1. Page 1 creates and joins the session
    await page1.goto('/');
    await page1.fill('#new-session-name', SESSION_ID);
    await page1.click('button:has-text("Create Session")');
    await expect(page1.locator('#terminal-view')).toBeVisible();
    await expect(page1.locator('#current-session-id')).toHaveText(SESSION_ID);

    // 2. Page 2 goes to dashboard and should see the session via SSE
    await page2.goto('/');
    const sessionCard = page2.locator(`#session-list div.group:has-text("${SESSION_ID}")`);
    await expect(sessionCard).toBeVisible();

    // 3. Page 2 deletes the session
    page2.on('dialog', dialog => dialog.accept());
    // Handle the alert on page 1 that appears when session is deleted
    page1.on('dialog', dialog => dialog.accept());
    
    const deleteButton = sessionCard.locator('button');
    await deleteButton.click();

    // 4. Page 2: Session card should disappear
    await expect(sessionCard).not.toBeVisible();

    // 5. Page 1: Should be kicked back to dashboard automatically via SSE signal
    await expect(page1.locator('#dashboard')).toBeVisible();
    await expect(page1.locator('#terminal-view')).toBeHidden();
  });

  test('should sync session deletion when shell process exits', async ({ context }) => {
    const page1 = await context.newPage();
    const page2 = await context.newPage();
    const SESSION_ID = `exit-sync-${Date.now()}`;

    // 1. Page 1 creates and joins the session
    await page1.goto('/');
    await page1.fill('#new-session-name', SESSION_ID);
    await page1.click('button:has-text("Create Session")');
    await expect(page1.locator('#terminal-view')).toBeVisible();

    // 2. Page 2 goes to dashboard and verifies session is visible
    await page2.goto('/');
    const sessionCard = page2.locator(`#session-list div.group:has-text("${SESSION_ID}")`);
    await expect(sessionCard).toBeVisible();

    // 3. Page 1 triggers shell exit
    // We send "exit\n" to the terminal. Xterm.js will forward it to WS.
    // The backend PTY will terminate, monitor_session will cleanup and broadcast.
    await page1.keyboard.type('exit');
    await page1.keyboard.press('Enter');

    // 4. Page 2: Session card should disappear automatically via SSE
    await expect(sessionCard).not.toBeVisible({ timeout: 10000 });

    // 5. Page 1: Should have been kicked back to dashboard
    await expect(page1.locator('#dashboard')).toBeVisible();
  });
});
