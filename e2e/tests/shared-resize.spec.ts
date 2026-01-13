import { test, expect } from '../fixtures';

test.describe('Shared PTY Dimensions (Smallest Screen Priority)', () => {
  test('should shrink terminal size to MIN of all connected clients', async ({ context }) => {
    const pageLarge = await context.newPage();
    const pageSmall = await context.newPage();
    const SESSION_ID = `resize-sync-min-${Date.now()}`;

    // 1. Setup Page Large (Desktop)
    await pageLarge.setViewportSize({ width: 1000, height: 800 });
    await pageLarge.goto('/');
    await pageLarge.fill('#new-session-name', SESSION_ID);
    await pageLarge.click('button:has-text("Create Session")');
    await expect(pageLarge.locator('#terminal-view')).toBeVisible();

    // Get initial size of large terminal
    const sizeLargeInitial = await pageLarge.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    console.log(`Large initial size: ${sizeLargeInitial.cols}x${sizeLargeInitial.rows}`);

    // 2. Setup Page Small (Mobile)
    await pageSmall.setViewportSize({ width: 400, height: 300 });
    await pageSmall.goto('/');
    
    // Join the existing session
    const sessionCard = pageSmall.locator(`#session-list div.group:has-text("${SESSION_ID}")`);
    await expect(sessionCard).toBeVisible();
    await sessionCard.click();
    await expect(pageSmall.locator('#terminal-view')).toBeVisible();

    // Wait for terminal to be initialized on small page
    await pageSmall.waitForFunction(() => window.term !== null);

    // 3. Verify Shared Dimensions (Both should be small)
    // We wait for the large page to decrease its rows/cols
    await pageLarge.waitForFunction((initial) => {
      return window.term.rows < initial.rows;
    }, sizeLargeInitial, { timeout: 15000 });

    const sizeLargeFinal = await pageLarge.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    const sizeSmallFinal = await pageSmall.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });

    console.log(`Large final size: ${sizeLargeFinal.cols}x${sizeLargeFinal.rows}`);
    console.log(`Small final size: ${sizeSmallFinal.cols}x${sizeSmallFinal.rows}`);

    // Large terminal should have SHRUNK to match the small one
    expect(sizeLargeFinal.cols).toBeLessThan(sizeLargeInitial.cols);
    expect(sizeLargeFinal.rows).toBeLessThan(sizeLargeInitial.rows);

    // Both should be equal to the small dimensions
    expect(sizeSmallFinal.cols).toBe(sizeLargeFinal.cols);
    expect(sizeSmallFinal.rows).toBe(sizeLargeFinal.rows);
  });

  test('should shrink terminal size if a new smaller client joins', async ({ context }) => {
    const pageLarge = await context.newPage();
    const pageSmall = await context.newPage();
    const SESSION_ID = `shrink-sync-${Date.now()}`;

    // 1. Large client starts session
    await pageLarge.setViewportSize({ width: 1000, height: 800 });
    await pageLarge.goto('/');
    await pageLarge.fill('#new-session-name', SESSION_ID);
    await pageLarge.click('button:has-text("Create Session")');
    await pageLarge.waitForSelector('#terminal-view', { state: 'visible' });

    // Wait for terminal to be ready
    await pageLarge.waitForFunction(() => window.term !== null && window.term.cols > 0);

    const sizeLargeInitial = await pageLarge.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    console.log(`Large initial size: ${sizeLargeInitial.cols}x${sizeLargeInitial.rows}`);

    // 2. Small client joins
    await pageSmall.setViewportSize({ width: 400, height: 300 });
    await pageSmall.goto('/');
    await pageSmall.locator(`#session-list div.group:has-text("${SESSION_ID}")`).click();
    await expect(pageSmall.locator('#terminal-view')).toBeVisible();

    // 3. Both should sync to the new MIN (Small size)
    await pageLarge.waitForFunction((initial) => {
      return window.term.cols < initial.cols;
    }, sizeLargeInitial, { timeout: 15000 });

    const sizeSmallFinal = await pageSmall.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    const sizeLargeFinal = await pageLarge.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });

    expect(sizeSmallFinal.cols).toBe(sizeLargeFinal.cols);
    expect(sizeSmallFinal.rows).toBe(sizeLargeFinal.rows);
    expect(sizeLargeFinal.cols).toBeLessThan(sizeLargeInitial.cols);
  });
});