import { test, expect } from '../fixtures';

test.describe('IME Context Injection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    const SESSION_NAME = `ime-context-${Date.now()}`;
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create Session")');
    await page.waitForSelector('#terminal-view', { state: 'visible' });
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);
  });

  test('should inject terminal content into textarea on compositionstart', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const term = (window as any).term;
      const textarea = document.querySelector('.xterm-helper-textarea') as HTMLTextAreaElement;
      
      // 1. Simulate existing text in terminal
      term.write('Hello World');
      
      // Wait for write to process
      await new Promise(r => setTimeout(r, 100));
      
      // Ensure textarea is empty (default state when screenReaderMode: false)
      textarea.value = '';
      
      // 2. Trigger compositionstart
      textarea.dispatchEvent(new CompositionEvent('compositionstart', { data: '' }));
      
      // 3. Return the value to check in Node context
      return textarea.value;
    });
    
    expect(result).toContain('Hello World');
  });
});
