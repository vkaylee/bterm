import { test, expect } from '../fixtures';

test.describe('Terminal Layout', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        // Create/Join a session
        await page.fill('#new-session-name', 'layout-test');
        await page.click('button:has-text("Create")');
        await page.waitForSelector('.xterm-screen');
    });

    test('should fill the available vertical space below header', async ({ page }) => {
        // Get dimensions
        const viewportSize = page.viewportSize();
        if (!viewportSize) throw new Error('Viewport size is null');

        const headerBox = await page.locator('header').boundingBox();
        const terminalViewBox = await page.locator('#terminal-view').boundingBox();

        if (!headerBox || !terminalViewBox) throw new Error('Elements not found');

        // Check width
        expect(terminalViewBox.width).toBe(viewportSize.width);

        // Check height
        // The terminal view should start right after the header
        expect(terminalViewBox.y).toBeCloseTo(headerBox.y + headerBox.height, 1);
        
        // The terminal view height + header height should roughly equal viewport height
        // (allowing for small rounding differences)
        expect(terminalViewBox.height + headerBox.height).toBeCloseTo(viewportSize.height, 2);
    });

    test('canvas should occupy almost all of the terminal container', async ({ page }) => {
        // Wait for fit addon to do its job
        await page.waitForTimeout(500);

        const terminalBox = await page.locator('#terminal').boundingBox();
        const screenBox = await page.locator('.xterm-screen').boundingBox();

        if (!terminalBox || !screenBox) throw new Error('Elements not found');

        // The xterm screen usually has a scrollbar or slight character alignment padding,
        // but it should be very close to the container size.
        
        // Calculate usage ratio
        const widthRatio = screenBox.width / terminalBox.width;
        const heightRatio = screenBox.height / terminalBox.height;

        console.log(`Width usage: ${widthRatio * 100}%`);
        console.log(`Height usage: ${heightRatio * 100}%`);

        // We expect it to use at least 95% of the available space
        // (It might be less if the font size is large and the window is small, leaving a gap)
        expect(widthRatio).toBeGreaterThan(0.90); 
        expect(heightRatio).toBeGreaterThan(0.90);
    });

    test('terminal should resize when window resizes', async ({ page }) => {
        await page.setViewportSize({ width: 1024, height: 768 });
        await page.waitForTimeout(500);
        
        const initialBox = await page.locator('.xterm-screen').boundingBox();
        
        // Resize window
        await page.setViewportSize({ width: 800, height: 600 });
        await page.waitForTimeout(1000); // Wait for debounce/resize observer

        const newBox = await page.locator('.xterm-screen').boundingBox();

        if (!initialBox || !newBox) throw new Error('Boxes not found');

        expect(newBox.width).toBeLessThan(initialBox.width);
        expect(newBox.height).toBeLessThan(initialBox.height);
    });
});
