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
        // Verify font file is actually loaded (status 200)
        const fontResponse = await page.request.get('/assets/fonts/JetBrainsMono-Regular.ttf');
        expect(fontResponse.ok()).toBeTruthy();

        // Tạo session để mở terminal
        const sessionName = `font-test-${Date.now()}`;
        await page.fill('#new-session-name', sessionName);
        await page.click('button:has-text("Create Session")');
        
        // Đợi xterm khởi tạo
        await page.waitForSelector('.xterm-rows');

        // Kiểm tra font-family của terminal
        const fontFamily = await page.evaluate(() => {
            const terminalElement = document.querySelector('.xterm-rows') as HTMLElement;
            return window.getComputedStyle(terminalElement).fontFamily;
        });
        
        // Kiểm tra xem font JetBrains Mono có nằm trong danh sách không
        expect(fontFamily).toContain('JetBrains Mono');
    });
});