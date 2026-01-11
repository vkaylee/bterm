import { test, expect } from '@playwright/test';

test.describe('Assets & UI Style', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:3000');
    });

    test('should load local assets successfully (no CDN)', async ({ page }) => {
        // Kiểm tra script tailwind nội bộ
        const tailwindScript = await page.locator('script[src="./assets/tailwindcss.js"]');
        await expect(tailwindScript).toBeAttached();

        // Kiểm tra tệp font JetBrains Mono
        const fontResponse = await page.request.get('http://localhost:3000/assets/fonts/JetBrainsMono-Regular.ttf');
        expect(fontResponse.ok()).toBeTruthy();
    });

    test('should apply JetBrains Mono font and correct font size to terminal', async ({ page }) => {
        // Tạo session để mở terminal
        const sessionName = `font-test-${Date.now()}`;
        await page.fill('#new-session-name', sessionName);
        await page.click('button:has-text("Create")');
        await page.click(`text=${sessionName}`);

        // Đợi xterm khởi tạo
        await page.waitForSelector('.xterm-rows');

        // Kiểm tra font-family của terminal
        const fontFamily = await page.evaluate(() => {
            const terminalElement = document.querySelector('.xterm-rows');
            return window.getComputedStyle(terminalElement).fontFamily;
        });
        
        // Kiểm tra xem font JetBrains Mono có nằm trong danh sách không
        expect(fontFamily).toContain('JetBrains Mono');
    });
});
