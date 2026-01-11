import { test, expect } from '../fixtures';

test.describe('Terminal Renderer Strategy', () => {
  
  test('should use Canvas renderer by default', async ({ page }, testInfo) => {
    page.on('console', msg => console.log(`[BROWSER] ${msg.type()}: ${msg.text()}`));
    // 1. Navigate to the app
    await page.goto('');
    
    // 2. Create a session to ensure terminal is initialized
    await page.locator('#new-session-name').fill('canvas-test-session');
    await page.locator('button', { hasText: 'Create Session' }).click();
    
    // 3. Wait for terminal to be visible
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForSelector('.xterm-screen'); 

    // Wait for renderer to be injected
    await page.waitForFunction(() => {
        // @ts-ignore
        const term = window.term;
        if (!term || !term._core || !term._core._renderService) return false;
        const renderer = term._core._renderService._renderer;
        // It might be a direct object or wrapped in an Observable (_value)
        return !!renderer && (renderer._value !== null);
    }, { timeout: 5000 });

    // 4. Verify renderer type by checking DOM structure
    const renderType = await page.evaluate(() => {
        // @ts-ignore
        const term = window.term;
        if (!term) return 'NoTerminal';
        
        const textCanvas = document.querySelector('.xterm-text-layer');
        const canvasElements = document.querySelectorAll('.xterm-screen canvas');
        
        if (textCanvas || canvasElements.length > 0) {
             // If there are canvas elements in xterm-screen, it's likely Canvas/WebGL renderer
             // DOM renderer might use canvas for cursor/link/selection?
             // But xterm-addon-canvas uses a specific text layer canvas.
             return 'Canvas';
        }
        return 'DOM';
    });

    console.log(`Detected Render Type for ${testInfo.project.name}:`, renderType);

    const isCanvas = renderType === 'Canvas';
    
    // Only expect Canvas if the project is Canvas
    if (testInfo.project.name === 'Canvas') {
        expect(isCanvas).toBe(true);
    } else {
        expect(isCanvas).toBe(false);
    }
  });

  test('should fallback to DOM renderer when Canvas is not supported', async ({ page }) => {
    // 1. Mock Canvas failure BEFORE page load
    await page.addInitScript(() => {
        const originalGetContext = HTMLCanvasElement.prototype.getContext;
        // @ts-ignore
        HTMLCanvasElement.prototype.getContext = function(type, ...args) {
            if (type === '2d') {
                return null;
            }
            return originalGetContext.call(this, type, ...args);
        };
    });

    // 2. Navigate to the app
    await page.goto('/');
    
    // 3. Create a session
    await page.locator('#new-session-name').fill('fallback-test-session');
    await page.locator('button', { hasText: 'Create Session' }).click();
    
    // 4. Wait for terminal
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForSelector('.xterm-screen');

    // 5. Verify renderer type
    const rendererName = await page.evaluate(() => {
        // @ts-ignore
        const term = window.term;
        if (!term) return 'NoTerminal';
        
        // @ts-ignore
        const renderer = term._core._renderService._renderer;
        return renderer.constructor.name;
    });

    expect(rendererName).not.toBe('CanvasRenderer');
  });

});
