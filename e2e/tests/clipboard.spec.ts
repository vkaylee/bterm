import { test, expect } from '../fixtures';

test.describe('Clipboard Interaction', () => {
    const SESSION_NAME = `clip-session-${Date.now()}`;

    test.beforeEach(async ({ page }) => {
        // Grant clipboard permissions
        await page.context().grantPermissions(['clipboard-read', 'clipboard-write']);
        
        await page.goto('/');
        await page.fill('#new-session-name', SESSION_NAME);
        // Wait for response to ensure session is created
        const [response] = await Promise.all([
          page.waitForResponse(response => response.url().includes('/api/sessions') && response.request().method() === 'POST'),
          page.click('button:has-text("Create Session")')
        ]);
        expect(response.ok()).toBeTruthy();
        
        await expect(page.locator('#terminal-view')).toBeVisible();
        await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);
    });

    test.afterEach(async ({ page }) => {
        const isDashboardVisible = await page.locator('#dashboard').isVisible();
        if (!isDashboardVisible) {
            await page.click('button[title="Exit Session"]');
        }
        const sessionCard = page.locator(`#session-list div.group:has-text("${SESSION_NAME}")`);
        if (await sessionCard.isVisible()) {
            const deleteButton = sessionCard.locator('button', { has: page.locator('svg') });
            await deleteButton.click();
        }
    });

    test('Ctrl+C should copy text when selected', async ({ page }) => {
        // 1. Output some text
        await page.keyboard.type('echo "SELECT_THIS_TEXT"');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500); // Wait for output

        // 2. Select text using xterm API
        await page.evaluate(() => {
            window.term.selectAll();
        });

        // 3. Press Ctrl+C
        await page.keyboard.press('Control+C');

        // 4. Check clipboard
        const clipText = await page.evaluate(() => navigator.clipboard.readText());
        // Note: selectAll includes prompts and everything, so we check inclusion
        expect(clipText).toContain('SELECT_THIS_TEXT');
    });

    test('Ctrl+C should send SIGINT when NO text selected', async ({ page }) => {
        await page.evaluate(() => window.term.clearSelection());

        // Start a blocking process
        await page.keyboard.type('cat');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(200);

        // Setup output capture to verify we get back control
        await page.evaluate(() => {
            (window as any).termOutput = '';
            const original = window.term.write.bind(window.term);
            window.term.write = (data: any) => {
                let decoded = typeof data === 'string' ? data : new TextDecoder().decode(data);
                (window as any).termOutput += decoded;
                original(data);
            };
        });

        // Send Ctrl+C (SIGINT)
        await page.keyboard.press('Control+C');
        await page.waitForTimeout(500);

        // Now verify we can run a command (which implies `cat` was interrupted)
        await page.keyboard.type('echo ALIVE');
        await page.keyboard.press('Enter');

        await page.waitForFunction(() => 
            (window as any).termOutput && (window as any).termOutput.includes('ALIVE'),
            { timeout: 5000 }
        );
    });

    test('Ctrl+V should paste text', async ({ page }) => {
        const PASTE_DATA = 'MAGIC_PASTE_' + Date.now();
        await page.evaluate((text) => navigator.clipboard.writeText(text), PASTE_DATA);

        // Setup capture
        await page.evaluate(() => {
            (window as any).termOutput = '';
            const original = window.term.write.bind(window.term);
            window.term.write = (data: any) => {
                let decoded = typeof data === 'string' ? data : new TextDecoder().decode(data);
                (window as any).termOutput += decoded;
                original(data);
            };
        });

        // Focus terminal and Paste
        await page.click('#terminal');
        await page.keyboard.press('Control+V');
        
        // Verify the text appears in the terminal (as echoed input)
        // We might need to wait a bit
        await page.waitForFunction((expected) => 
            (window as any).termOutput && (window as any).termOutput.includes(expected),
            PASTE_DATA,
            { timeout: 5000 }
        );
    });
});
