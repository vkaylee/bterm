import { test, expect } from '../fixtures';

test.describe('Terminal Renderer Strategy', () => {
  
  test('should support WebGL renderer when requested', async ({ page }, testInfo) => {
    // 1. Navigate to the app
    await page.goto('');
    
    // 2. Create a session to ensure terminal is initialized
    await page.locator('#new-session-name').fill('webgl-test-session');
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

    // 4. Verify renderer type
    const rendererInfo = await page.evaluate(() => {
        // @ts-ignore
        const term = window.term;
        if (!term) return { type: 'NoTerminal' };
        
        // @ts-ignore
        const renderService = term._core._renderService;
        let renderer = renderService._renderer;
        
        // Handle Observable wrapper if present
        if (renderer && renderer._value) {
            renderer = renderer._value;
        }
        
        if (!renderer) return { type: 'NoRenderer' };
        
        return {
            rendererConstructor: renderer.constructor ? renderer.constructor.name : 'NoConstructor',
            isMock: !!renderer.terminal || !!renderer._terminal,
            keys: Object.keys(renderer)
        };
    });

    // Check if it's WebGL based on constructor name OR our mock indicator
    const isWebgl = rendererInfo.rendererConstructor === 'WebglRenderer' || 
                   rendererInfo.rendererConstructor === 'MockWebglRenderer' ||
                   rendererInfo.isMock;
    
    // Only expect WebGL if the project is WebGL
    if (testInfo.project.name === 'WebGL') {
        expect(isWebgl).toBe(true);
    } else {
        expect(isWebgl).toBe(false);
    }
  });

  test('should fallback to DOM renderer when WebGL is not supported', async ({ page }) => {
    // 1. Mock WebGL failure BEFORE page load
    await page.addInitScript(() => {
        const originalGetContext = HTMLCanvasElement.prototype.getContext;
        // @ts-ignore
        HTMLCanvasElement.prototype.getContext = function(type, ...args) {
            if (type === 'webgl' || type === 'webgl2') {
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
        // DOM renderer usually doesn't have 'WebglRenderer' name
        return renderer.constructor.name;
    });

    expect(rendererName).not.toBe('WebglRenderer');
  });

});
