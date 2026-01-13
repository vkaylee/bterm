import { test, expect } from '../fixtures';

test.describe('Renderer Optimization (Flicker Prevention)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    const SESSION_NAME = `render-opt-${Date.now()}`;
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create Session")');
    await page.waitForSelector('.xterm-screen', { state: 'visible' });
  });

  test('should enforce integer pixel dimensions (Integer Snapping)', async ({ page }) => {
    // 1. Chờ cho đến khi terminal được render hoàn chỉnh
    await page.waitForFunction(() => {
      const el = document.getElementById('terminal');
      return el && el.clientWidth > 0;
    });

    // 2. Ép trình duyệt resize để kích hoạt logic fit
    await page.setViewportSize({ width: 805, height: 605 }); // Số lẻ để test snapping
    await page.waitForTimeout(500); // Chờ throttledFit

    // 3. Lấy kích thước thực tế
    const dimensions = await page.evaluate(() => {
      const el = document.getElementById('terminal');
      return {
        width: el.getBoundingClientRect().width,
        height: el.getBoundingClientRect().height
      };
    });

    console.log(`Terminal dimensions after resize: ${dimensions.width}x${dimensions.height}`);

    // 4. Xác nhận chiều rộng và chiều cao là số nguyên
    // Chúng ta cho phép sai số cực nhỏ 0.01 do cơ chế làm tròn của trình duyệt khi tính zoom
    expect(dimensions.width % 1).toBeLessThan(0.1);
    expect(dimensions.height % 1).toBeLessThan(0.1);
  });

  test('should remain stable during high-frequency data streaming', async ({ page }) => {
    // Giả lập Gemini CLI output nhanh
    await page.evaluate(async () => {
      const term = (window as any).term;
      const data = "Streaming text chunk... ".repeat(50) + "\n";
      
      // Ghi 100 lần liên tục cực nhanh
      for (let i = 0; i < 100; i++) {
        term.write(data);
      }
      
      // Chờ một chút để RAF thực thi
      await new Promise(r => requestAnimationFrame(r));
    });

    // Xác nhận terminal vẫn phản hồi và cuộn xuống cuối
    const isAtBottom = await page.evaluate(() => {
      const term = (window as any).term;
      return term.buffer.active.viewportY === term.buffer.active.baseY;
    });
    
    expect(isAtBottom).toBe(true);
  });

  test('should not have will-change: transform (prevent sub-pixel shimmering)', async ({ page }) => {
    const willChange = await page.evaluate(() => {
      const el = document.getElementById('terminal');
      return window.getComputedStyle(el).willChange;
    });
    
    // Xác nhận đã gỡ bỏ will-change để tránh lỗi render sub-pixel trên một số trình duyệt
    expect(willChange).not.toBe('transform');
  });
});
