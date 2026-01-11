import { test, expect } from '@playwright/test';

test.describe('Mobile UI & Keyboard Overlay', () => {
  const SESSION_NAME = `mobile-test-${Date.now()}`;

  test.beforeEach(async ({ page }) => {
    // Emulate mobile device
    await page.setViewportSize({ width: 375, height: 667 });
    
    await page.goto('/');
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create")');
    
    // Auto-join should have happened
    await page.waitForSelector('#terminal-view');
  });

  test('should show control bar with 2 rows on mobile', async ({ page }) => {
    const controlBar = page.locator('#control-bar');
    await expect(controlBar).toBeVisible();
    
    // Check for 2 grid rows
    const rows = controlBar.locator('.grid');
    await expect(rows).toHaveCount(2);
  });

  test('should handle sticky Ctrl + c combination', async ({ page }) => {
    // 1. Click Ctrl button using dispatchEvent to ensure onclick is triggered
    const ctrlBtn = page.locator('#btn-ctrl-key');
    await ctrlBtn.dispatchEvent('click');
    
    // Verify it turns active via data-attribute
    await expect(ctrlBtn).toHaveAttribute('data-active', 'true');

    // 2. Mock WebSocket send to capture the final data
    const sentData = await page.evaluate(async () => {
      return new Promise((resolve) => {
        const originalSend = (window as any).ws.send;
        (window as any).ws.send = (msg: string) => {
          const parsed = JSON.parse(msg);
          if (parsed.type === 'Input') {
            (window as any).ws.send = originalSend; // restore
            resolve(parsed.data);
          }
          originalSend.apply((window as any).ws, [msg]);
        };
        
        // Trigger a 'c' input through xterm.js
        (window as any).term._core._onData.fire('c');
      });
    });

    // 3. Verify that 'c' was transformed to \x03 (Ctrl+C)
    expect(sentData).toBe('\x03');
    
    // 4. Verify Ctrl button returns to normal
    await expect(ctrlBtn).toHaveAttribute('data-active', 'false');
  });

  test('should offset control bar when visual viewport is smaller (keyboard simulation)', async ({ page }) => {
    const controlBar = page.locator('#control-bar');
    
    // Simulate keyboard opening by mocking visualViewport properties
    await page.evaluate(() => {
      const originalViewport = window.visualViewport;
      if (!originalViewport) return;

      // We mock the height to be smaller (as if keyboard took 300px)
      // and offsetTop to be 0 (keyboard at bottom)
      const mockHeight = window.innerHeight - 300;
      
      // Since visualViewport is read-only, we might need to call the update function directly 
      // with mocked values or use Object.defineProperty if we want the event listener to pick it up.
      
      // Let's redefine the property for the test
      Object.defineProperty(window, 'visualViewport', {
        value: {
          height: mockHeight,
          offsetTop: 0,
          addEventListener: originalViewport.addEventListener.bind(originalViewport),
          removeEventListener: originalViewport.removeEventListener.bind(originalViewport),
        },
        configurable: true
      });

      // Manually trigger the update function that's defined in index.html
      // We expect the function to be in the global scope or accessible.
      // In index.html it is defined in the script tag, so it's global.
      (window as any).updateVisualViewport();
    });

    // Check if transform is applied: translateY(-300px)
    const transform = await controlBar.evaluate(el => el.style.transform);
    expect(transform).toBe('translateY(-300px)');
  });

  test('should reset control bar when visual viewport returns to normal', async ({ page }) => {
    const controlBar = page.locator('#control-bar');
    
    await page.evaluate(() => {
      // Simulate keyboard closing
      Object.defineProperty(window, 'visualViewport', {
        value: {
          height: window.innerHeight,
          offsetTop: 0,
        },
        configurable: true
      });
      (window as any).updateVisualViewport();
    });

    const transform = await controlBar.evaluate(el => el.style.transform);
    expect(transform).toBe('none');
  });
});
