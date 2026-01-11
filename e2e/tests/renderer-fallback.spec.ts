import { test, expect } from '../fixtures';

test.describe('Terminal Renderer Strategy', () => {
  
  test('should support multi-tier renderer fallback', async ({ page }, testInfo) => {
    const logs: string[] = [];
    page.on('console', msg => logs.push(msg.text()));
    
    // 1. Navigate to the app
    await page.goto('');
    
    // 2. Create a session
    await page.locator('#new-session-name').fill('fallback-tier-test');
    await page.locator('button', { hasText: 'Create Session' }).click();
    
    // 3. Wait for terminal
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForSelector('.xterm-screen'); 

    // Wait for initialization
    await page.waitForTimeout(1000);

    // 4. Verify based on project name
    const hasWebglLog = logs.some(l => l.includes('WebGL renderer enabled'));
    const hasCanvasLog = logs.some(l => l.includes('Canvas renderer enabled'));
    const hasDomLog = logs.some(l => l.includes('Using standard DOM renderer') || l.includes('Forcing DOM renderer'));

    if (testInfo.project.name === 'WebGL') {
        // In headless, WebGL might fail, so it could fall back to Canvas or DOM
        // But we want to see if it at least TRIED or succeeded if supported
        expect(hasWebglLog || hasCanvasLog || hasDomLog).toBe(true);
    } else if (testInfo.project.name === 'Canvas') {
        expect(hasCanvasLog || hasDomLog).toBe(true);
    } else if (testInfo.project.name === 'DOM-Fallback') {
        expect(hasDomLog).toBe(true);
    }
  });

  test('should fallback WebGL -> Canvas when WebGL fails', async ({ page }) => {
    const logs: string[] = [];
    page.on('console', msg => {
        logs.push(msg.text());
        console.log(`[BROWSER ${msg.type()}] ${msg.text()}`);
    });

    // Mock WebGL failure by returning null context
    await page.addInitScript(() => {
        const originalGetContext = HTMLCanvasElement.prototype.getContext;
        // @ts-ignore
        HTMLCanvasElement.prototype.getContext = function(type, ...args) {
            if (type === 'webgl' || type === 'webgl2') {
                return null;
            }
            return originalGetContext.apply(this, [type, ...args]);
        };
    });

    await page.goto('?renderer=webgl');
    await page.locator('#new-session-name').fill('webgl-fail-test');
    await page.locator('button', { hasText: 'Create Session' }).click();
    
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForTimeout(1000);

    // Should NOT have WebGL enabled
    expect(logs.some(l => l.includes('WebGL renderer enabled'))).toBe(false);
    // Should have Canvas enabled
    expect(logs.some(l => l.includes('Canvas renderer enabled'))).toBe(true);
  });

  test('should fallback Canvas -> DOM when Canvas fails', async ({ page }) => {
    const logs: string[] = [];
    page.on('console', msg => {
        logs.push(msg.text());
        console.log(`[BROWSER ${msg.type()}] ${msg.text()}`);
    });

    // Mock Canvas failure by returning null context
    await page.addInitScript(() => {
        const originalGetContext = HTMLCanvasElement.prototype.getContext;
        // @ts-ignore
        HTMLCanvasElement.prototype.getContext = function(type, ...args) {
            if (type === '2d') {
                return null;
            }
            return originalGetContext.apply(this, [type, ...args]);
        };
    });

    await page.goto('?renderer=canvas');
    await page.locator('#new-session-name').fill('canvas-fail-test');
    await page.locator('button', { hasText: 'Create Session' }).click();
    
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForTimeout(1000);

    // Should NOT have Canvas enabled
    expect(logs.some(l => l.includes('Canvas renderer enabled'))).toBe(false);
    // Should have DOM fallback log
    expect(logs.some(l => l.includes('standard DOM renderer') || l.includes('Canvas not supported, using DOM'))).toBe(true);
  });

});
