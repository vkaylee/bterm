import { test, expect } from '../fixtures';

test.describe('Shared PTY Dimensions (Approach C)', () => {
  test('should keep terminal size at MAX of all connected clients', async ({ context }) => {
    const pageLarge = await context.newPage();
    const pageSmall = await context.newPage();
    const SESSION_ID = `resize-sync-${Date.now()}`;

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

    // 3. Verify Shared Dimensions
    // Give some time for sync
    await pageLarge.waitForTimeout(1000);

    const sizeLargeFinal = await pageLarge.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    const sizeSmallFinal = await pageSmall.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });

    console.log(`Large final size: ${sizeLargeFinal.cols}x${sizeLargeFinal.rows}`);
    console.log(`Small final size: ${sizeSmallFinal.cols}x${sizeSmallFinal.rows}`);

    // Large terminal should NOT have shrunk
    expect(sizeLargeFinal.cols).toBeGreaterThanOrEqual(sizeLargeInitial.cols);
    expect(sizeLargeFinal.rows).toBeGreaterThanOrEqual(sizeLargeInitial.rows);

    // Small terminal should have been FORCED to the large size by the server
    expect(sizeSmallFinal.cols).toBe(sizeLargeFinal.cols);
    expect(sizeSmallFinal.rows).toBe(sizeLargeFinal.rows);

    // 4. Verify scrolling/overflow on small page
    // Verify terminal is scrollable or at least larger than viewport (Approach C)
    const isLargerThanViewport = await pageSmall.evaluate(() => {
      const view = document.getElementById('terminal-view');
      const terminal = document.getElementById('terminal');
      return terminal.clientWidth > view.clientWidth || terminal.clientHeight > view.clientHeight ||
             window.getComputedStyle(view).overflow === 'auto';
    });
    expect(isLargerThanViewport).toBe(true);
  });

  test('should grow terminal size if a new larger client joins', async ({ context }) => {
    const pageSmall = await context.newPage();
    const pageLarge = await context.newPage();
    const SESSION_ID = `grow-sync-${Date.now()}`;

    // 1. Small client starts session
    await pageSmall.setViewportSize({ width: 400, height: 300 });
    await pageSmall.goto('/');
    await pageSmall.fill('#new-session-name', SESSION_ID);
    await pageSmall.click('button:has-text("Create Session")');
    await pageSmall.waitForSelector('#terminal-view', { state: 'visible' });

    // Wait for terminal to be ready
    await pageSmall.waitForFunction(() => window.term !== null && window.term.cols > 0);

    const sizeSmallInitial = await pageSmall.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    console.log(`Small initial size: ${sizeSmallInitial.cols}x${sizeSmallInitial.rows}`);

    // 2. Large client joins
    await pageLarge.setViewportSize({ width: 1000, height: 800 });
    await pageLarge.goto('/');
    await pageLarge.locator(`#session-list div.group:has-text("${SESSION_ID}")`).click();
    await expect(pageLarge.locator('#terminal-view')).toBeVisible();

    // 3. Both should sync to the new MAX (Large size)
    // We wait for the small page to increase its columns
    await pageSmall.waitForFunction((initial) => {
      return window.term.cols > initial.cols;
    }, sizeSmallInitial, { timeout: 15000 });

    const sizeSmallFinal = await pageSmall.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });
    const sizeLargeFinal = await pageLarge.evaluate(() => {
      return { cols: window.term.cols, rows: window.term.rows };
    });

    expect(sizeSmallFinal.cols).toBe(sizeLargeFinal.cols);
    expect(sizeSmallFinal.rows).toBe(sizeLargeFinal.rows);
    expect(sizeSmallFinal.cols).toBeGreaterThan(sizeSmallInitial.cols);
  });
});