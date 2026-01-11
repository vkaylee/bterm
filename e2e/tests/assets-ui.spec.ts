import { test, expect } from '../fixtures';

test.describe('Assets UI', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
    });

    test('should load local assets successfully (no CDN)', async ({ page }) => {
        // Kiểm tra xem các file script và link đã load đúng (không có 404)
        const scripts = await page.locator('script[src]').all();
        for (const script of scripts) {
            const src = await script.getAttribute('src');
            if (src && !src.startsWith('http')) {
                const response = await page.request.get(src);
                expect(response.status()).toBe(200);
            }
        }
    });

  test('should apply JetBrains Mono font and correct font size to terminal', async ({ page }) => {
    // Need to create/join a session to initialize terminal
    const SESSION_NAME = 'font-test';
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create")');

    // Wait for terminal to be initialized
    await page.waitForFunction(() => (window as any).term !== null);
    
    const terminalData = await page.evaluate(() => {
      const term = (window as any).term;
      return {
        fontFamily: term.options.fontFamily,
        fontSize: term.options.fontSize,
      };
    });

    expect(terminalData.fontFamily).toContain('JetBrains Mono');
    expect(terminalData.fontSize).toBe(16);
  });
});