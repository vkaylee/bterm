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
    expect(terminalData.fontSize).toBe(15);
  });

  test('should load Canvas addon script and have it available', async ({ page }) => {
    const responses: any[] = [];
    page.on('response', response => {
      if (response.url().includes('canvas')) {
        responses.push({ url: response.url(), status: response.status() });
      }
    });

    await page.goto('/');
    
    const scriptExists = await page.locator('script[src*="canvas"]').count();
    console.log('Script tags for canvas:', scriptExists);
    console.log('Canvas network responses:', responses);

    const globals = await page.evaluate(() => {
      return Object.keys(window);
    });
    const canvasGlobals = globals.filter(k => k.toLowerCase().includes('canvas'));
    console.log('Canvas related globals:', canvasGlobals);
    
    expect(scriptExists).toBe(1);
    expect(responses.length).toBeGreaterThan(0);
    expect(responses[0].status).toBe(200);
  });

  test('should use Canvas renderer for performance', async ({ page }) => {
    // Need to create/join a session
    const SESSION_NAME = 'canvas-perf-test';
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create")');

    // Wait for terminal
    await page.waitForFunction(() => (window as any).term !== null);
    
    // CanvasRenderer creates <canvas> elements inside .xterm-screen
    const canvasCount = await page.locator('.xterm-screen canvas').count();
    
    console.log('Canvas count in terminal:', canvasCount);
    // Canvas renderer should have multiple canvases (text, cursor, selection, link)
    expect(canvasCount).toBeGreaterThan(0); 
  });
});