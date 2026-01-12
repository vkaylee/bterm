import { test, expect } from '../fixtures';

test.describe('Mobile UI', () => {
  test.beforeEach(async ({ page }) => {
    const SESSION_NAME = `mobile-test-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
    // Emulate mobile device
    await page.setViewportSize({ width: 375, height: 667 });
    
    await page.goto('/');
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create Session")');
    
    // Auto-join should have happened
    await page.waitForSelector('#terminal-view', { state: 'visible', timeout: 10000 });
  });

  test('should hide control bar initially and show only when keyboard appears', async ({ page }) => {
    const controlBar = page.locator('#control-bar');
    
    // Initially hidden (since we don't simulate keyboard yet)
    await expect(controlBar).toBeHidden();
    
    // Simulate keyboard opening
    await page.evaluate(() => {
      Object.defineProperty(window, 'visualViewport', {
        value: {
          height: window.innerHeight - 300,
          offsetTop: 0,
          addEventListener: () => {},
        },
        configurable: true
      });
      (window as any).updateVisualViewport();
    });

    await expect(controlBar).toBeVisible();
    
    // Check for 2 grid rows
    const rows = controlBar.locator('.grid');
    await expect(rows).toHaveCount(2);

    // Simulate keyboard closing
    await page.evaluate(() => {
      Object.defineProperty(window, 'visualViewport', {
        value: {
          height: window.innerHeight,
          offsetTop: 0,
        },
        configurable: true
      });
      (window as any).updateVisualViewport();
    });

    await expect(controlBar).toBeHidden();
  });

  test('should adjust #app height when visualViewport height changes', async ({ page }) => {
    const initialHeight = await page.evaluate(() => window.innerHeight);
    const app = page.locator('#app');

    // Simulate keyboard opening (reducing viewport height)
    const keyboardHeight = 300;
    await page.evaluate((kHeight) => {
      Object.defineProperty(window, 'visualViewport', {
        value: {
          height: window.innerHeight - kHeight,
          offsetTop: 0,
          addEventListener: () => {},
        },
        configurable: true
      });
      (window as any).updateVisualViewport();
    }, keyboardHeight);

    const adjustedHeight = await app.evaluate((el) => parseFloat(el.style.height));
    expect(adjustedHeight).toBe(initialHeight - keyboardHeight);

    // Simulate keyboard closing
    await page.evaluate(() => {
      Object.defineProperty(window, 'visualViewport', {
        value: {
          height: window.innerHeight,
          offsetTop: 0,
        },
        configurable: true
      });
      (window as any).updateVisualViewport();
    });

    const finalHeight = await app.evaluate((el) => parseFloat(el.style.height));
    expect(finalHeight).toBe(initialHeight);
  });

  test('should disable screenReaderMode and set correct textarea attributes for IME support', async ({ page }) => {
    // 1. Check screenReaderMode option in xterm.js (Should be false now)
    const screenReaderMode = await page.evaluate(() => (window as any).term.options.screenReaderMode);
    expect(screenReaderMode).toBe(false);

    // 2. Check textarea attributes (crucial for mobile Vietnamese Telex/IME)
    const textarea = page.locator('.xterm-helper-textarea');
    await expect(textarea).toHaveAttribute('autocorrect', 'off');
    await expect(textarea).toHaveAttribute('autocapitalize', 'none');
    await expect(textarea).toHaveAttribute('spellcheck', 'false');
    await expect(textarea).toHaveAttribute('inputmode', 'text');
  });

  test('should handle sticky Ctrl + c combination when visible', async ({ page }) => {
    // 0. Force keyboard visible to interact with buttons
    await page.evaluate(() => {
      Object.defineProperty(window, 'visualViewport', {
        value: { height: window.innerHeight - 300, offsetTop: 0 },
        configurable: true
      });
      (window as any).updateVisualViewport();
    });

    // Ensure WebSocket is ready
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);

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

  test('should have scrollIntoView behavior on session input for mobile accessibility', async ({ page }) => {
    // Go back to dashboard to see the input
    await page.click('button[title="Exit Session"]');
    const input = page.locator('#new-session-name');
    const onfocus = await input.getAttribute('onfocus');
    expect(onfocus).toContain('scrollIntoView');
  });

  test('should use passive listeners and stable throttled updates for performance', async ({ page }) => {
    // Check if the source code contains passive: true registration
    const content = await page.content();
    expect(content).toContain('passive: true');

    // Verify throttledUpdate doesn't break app height update after rapid triggers
    const initialHeight = await page.evaluate(() => window.innerHeight);
    const app = page.locator('#app');

    // Trigger updateVisualViewport multiple times rapidly (simulating throttle stress)
    await page.evaluate((height) => {
      for (let i = 0; i < 10; i++) {
        // Mock visualViewport for each iteration
        Object.defineProperty(window, 'visualViewport', {
          value: { height: height - i, offsetTop: 0 },
          configurable: true
        });
        (window as any).updateVisualViewport();
      }
    }, initialHeight);

    // After microtask/animation frame, height should settle to the last value (initialHeight - 9)
    await page.waitForTimeout(100); 
    const finalHeight = await app.evaluate((el) => parseFloat(el.style.height));
    expect(finalHeight).toBe(initialHeight - 9);
  });

  test('should allow user scaling and have manipulation touch-action for dragging/zooming', async ({ page }) => {
    // 1. Check viewport meta tag for user-scalable
    const viewportMeta = await page.locator('meta[name="viewport"]').getAttribute('content');
    expect(viewportMeta).toContain('user-scalable=yes');

    // 2. Check touch-action on #terminal
    const touchAction = await page.locator('#terminal').evaluate((el) => window.getComputedStyle(el).touchAction);
    expect(touchAction).toBe('manipulation');
  });
});
