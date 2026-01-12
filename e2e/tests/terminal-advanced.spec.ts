import { test, expect } from '../fixtures';

test.describe('Advanced Terminal Features', () => {
  const SESSION_NAME = `adv-session-${Date.now()}`;

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create Session")');
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);
  });

  test.afterEach(async ({ page }) => {
    // Clean up session
    await page.goto('/');
    const sessionCard = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    if (await sessionCard.isVisible()) {
      page.on('dialog', d => d.accept());
      await sessionCard.locator('button', { has: page.locator('svg') }).click();
      await expect(sessionCard).toBeHidden();
    }
  });

  test('should resize PTY on window resize', async ({ page }) => {
    // 1. Initial size
    await page.setViewportSize({ width: 800, height: 600 });
    await page.waitForTimeout(1000); // Wait for fit

    // Function to get stty size from terminal output
    const getSttySize = async () => {
      await page.evaluate(() => { (window as any).terminalOutput = ''; });
      await page.keyboard.type('stty size');
      await page.keyboard.press('Enter');
      
          // Wait for output that looks like "rows cols"
      
          await page.waitForFunction(() => {
      
            const out = (window as any).terminalOutput;
      
            return out && /\d+ \d+/.test(out) && out.includes('stty size');
      
          }, { timeout: 10000 });
      
      
      
          const output = await page.evaluate(() => (window as any).terminalOutput);
      
      
      const match = output.match(/(\d+) (\d+)/);
      return match ? { rows: parseInt(match[1]), cols: parseInt(match[2]) } : null;
    };

    // Setup output capture
    await page.evaluate(() => {
      (window as any).terminalOutput = '';
      const originalWrite = window.term.write.bind(window.term);
      window.term.write = (data: any) => {
          let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
          (window as any).terminalOutput += decoded;
          originalWrite(data);
      };
    });

    const size1 = await getSttySize();
    expect(size1).not.toBeNull();
    console.log(`Initial size: ${size1?.rows}x${size1?.cols}`);

    // 2. Resize
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.waitForTimeout(1000); // Wait for debounce and fitAddon

    const size2 = await getSttySize();
    expect(size2).not.toBeNull();
    console.log(`New size: ${size2?.rows}x${size2?.cols}`);

    expect(size2!.cols).toBeGreaterThan(size1!.cols);
    expect(size2!.rows).toBeGreaterThan(size1!.rows);
  });

  test('should handle Ctrl+C to interrupt process', async ({ page }) => {
    // Setup output capture
    await page.evaluate(() => {
      (window as any).terminalOutput = '';
      const originalWrite = window.term.write.bind(window.term);
      window.term.write = (data: any) => {
          let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
          (window as any).terminalOutput += decoded;
          originalWrite(data);
      };
    });

    // Run a long command
    await page.keyboard.type('sleep 100');
    await page.keyboard.press('Enter');
    
    // Wait a bit to ensure it's running
    await page.waitForTimeout(1000);

    // Send Ctrl+C
    await page.keyboard.press('Control+C');

    // Verify it returns to prompt (usually contains $)
    // We can also echo something after Ctrl+C to be sure
    await page.keyboard.type('echo INTERRUPTED');
    await page.keyboard.press('Enter');

    await page.waitForFunction(() => {
        return (window as any).terminalOutput && (window as any).terminalOutput.includes('INTERRUPTED');
    }, { timeout: 10000 });
    
    const output = await page.evaluate(() => (window as any).terminalOutput);
    expect(output).toContain('INTERRUPTED');
  });

  test('should persist and reconnect to session on reload', async ({ page }) => {
    // Setup capture
    await page.evaluate(() => {
        (window as any).terminalOutput = '';
        const originalWrite = window.term.write.bind(window.term);
        window.term.write = (data: any) => {
            let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
            (window as any).terminalOutput += decoded;
            originalWrite(data);
        };
    });

    // Run a command
    await page.keyboard.type('echo RELOAD_TEST');
    await page.keyboard.press('Enter');

    // Wait for it to appear
    await page.waitForFunction(() => {
        return (window as any).terminalOutput && (window as any).terminalOutput.includes('RELOAD_TEST');
    }, { timeout: 10000 });

    // Reload the page
    await page.reload();
    
    await expect(page.locator('#dashboard')).toBeVisible();
    const sessionCard = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
    await expect(sessionCard).toBeVisible();
    
    // Re-join
    await sessionCard.click();
    await expect(page.locator('#terminal-view')).toBeVisible();
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);

    // Setup capture again after reload
    await page.evaluate(() => {
        (window as any).terminalOutput = '';
        const originalWrite = window.term.write.bind(window.term);
        window.term.write = (data: any) => {
            let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
            (window as any).terminalOutput += decoded;
            originalWrite(data);
        };
    });

    // Verify we can still send commands
    await page.keyboard.type('echo STILL_HERE');
    await page.keyboard.press('Enter');
    
    await page.waitForFunction(() => {
        return (window as any).terminalOutput && (window as any).terminalOutput.includes('STILL_HERE');
    }, { timeout: 10000 });
  });

  test('should have scrollback buffer', async ({ page }) => {
    // Initialize capture BEFORE sending the command
    await page.evaluate(() => {
        (window as any).terminalOutput = '';
        if (!window.term) return;
        const originalWrite = window.term.write.bind(window.term);
        window.term.write = (data: any) => {
            let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
            (window as any).terminalOutput = ((window as any).terminalOutput || '') + decoded;
            originalWrite(data);
        };
    });

    // Generate many lines
    await page.keyboard.type('for i in {1..200}; do echo "LINE $i"; done');
    await page.keyboard.press('Enter');

    // Wait for the last line to appear
    await page.waitForFunction(() => {
        const out = (window as any).terminalOutput;
        return out && out.includes('LINE 200');
    }, { timeout: 15000 });

    // Check xterm scroll position or buffer
    const scrollResult = await page.evaluate(() => {
        return {
            bufferLength: window.term.buffer.active.length,
            viewportY: window.term.buffer.active.viewportY
        };
    });

    expect(scrollResult.bufferLength).toBeGreaterThan(150);
    // If it scrolled, viewportY should be > 0 (assuming viewport is smaller than 200 lines)
    expect(scrollResult.viewportY).toBeGreaterThan(0);
  });

  test('should maintain isolation between multiple sessions', async ({ context }) => {
    const page1 = await context.newPage();
    const page2 = await context.newPage();
    const ID1 = `session-isolation-1-${Date.now()}`;
    const ID2 = `session-isolation-2-${Date.now()}`;

    // Setup isolation capture
    const setupCapture = async (page: any) => {
        await page.evaluate(() => {
            (window as any).terminalOutput = '';
            const originalWrite = window.term.write.bind(window.term);
            window.term.write = (data: any) => {
                let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
                (window as any).terminalOutput += decoded;
                originalWrite(data);
            };
        });
    };

    // 1. Join session 1
    await page1.goto('/');
    await page1.fill('#new-session-name', ID1);
    await page1.click('button:has-text("Create Session")');
    await expect(page1.locator('#terminal-view')).toBeVisible();
    await setupCapture(page1);

    // 2. Join session 2
    await page2.goto('/');
    await page2.fill('#new-session-name', ID2);
    await page2.click('button:has-text("Create Session")');
    await expect(page2.locator('#terminal-view')).toBeVisible();
    await setupCapture(page2);

    // 3. Send commands
    await page1.keyboard.type(`echo "SECRET_ONE"`);
    await page1.keyboard.press('Enter');
    
    await page2.keyboard.type(`echo "SECRET_TWO"`);
    await page2.keyboard.press('Enter');

    // 4. Verify isolation
    await page1.waitForFunction(() => (window as any).terminalOutput.includes('SECRET_ONE'), { timeout: 5000 });
    await page2.waitForFunction(() => (window as any).terminalOutput.includes('SECRET_TWO'), { timeout: 5000 });

    const out1 = await page1.evaluate(() => (window as any).terminalOutput);
    const out2 = await page2.evaluate(() => (window as any).terminalOutput);

    expect(out1).toContain('SECRET_ONE');
    expect(out1).not.toContain('SECRET_TWO');
    expect(out2).toContain('SECRET_TWO');
    expect(out2).not.toContain('SECRET_ONE');

    // Cleanup
    await page1.goto('/');
    await page2.goto('/');
    page1.on('dialog', d => d.accept());
    page2.on('dialog', d => d.accept());
    await page1.locator(`#session-list div.group:has-text("${ID1}")`).locator('button', { has: page1.locator('svg') }).click();
    await page2.locator(`#session-list div.group:has-text("${ID2}")`).locator('button', { has: page2.locator('svg') }).click();
  });

  test('should handle UTF-8 characters correctly', async ({ page }) => {
    const utf8String = 'Hello, 世界! 🚀 Testing UTF-8: 🔥❄️✨';
    
    await page.evaluate(() => {
        (window as any).terminalOutput = '';
        const originalWrite = window.term.write.bind(window.term);
        window.term.write = (data: any) => {
            let decoded = (typeof data === 'string') ? data : new TextDecoder().decode(data);
            (window as any).terminalOutput += decoded;
            originalWrite(data);
        };
    });

    await page.keyboard.type(`echo "${utf8String}"`);
    await page.keyboard.press('Enter');

    await page.waitForFunction((expected) => (window as any).terminalOutput.includes(expected), utf8String, { timeout: 5000 });
    
    const output = await page.evaluate(() => (window as any).terminalOutput);
    expect(output).toContain(utf8String);
  });
});
