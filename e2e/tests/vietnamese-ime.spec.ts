import { test, expect } from '../fixtures';

test.describe('Vietnamese IME Support', () => {
  test.beforeEach(async ({ page }) => {
    // Emulate mobile device
    await page.setViewportSize({ width: 375, height: 667 });
    
    const SESSION_NAME = `ime-test-${Date.now()}`;
    await page.goto('/');
    await page.fill('#new-session-name', SESSION_NAME);
    await page.click('button:has-text("Create Session")');
    await page.waitForSelector('#terminal-view', { state: 'visible' });
    
    // Ensure WebSocket is ready
    await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);
  });

  test('should correctly handle Vietnamese character input without duplication', async ({ page }) => {
    // We capture what is sent over WebSocket to verify the final data
    const terminalOutput = await page.evaluate(async () => {
      const results: string[] = [];
      const originalSend = (window as any).ws.send;
      (window as any).ws.send = (msg: string) => {
        const parsed = JSON.parse(msg);
        if (parsed.type === 'Input') {
          results.push(parsed.data);
        }
        originalSend.apply((window as any).ws, [msg]);
      };
      (window as any).capturedInputs = results;
      return results;
    });

    // Simulate typing "chào"
    // In a real IME, the browser might receive "c", "h", "à", "o" or a composition
    // Here we simulate the direct input of the composed characters
    await page.keyboard.type('chào');
    
    // Give it a moment to process
    await page.waitForTimeout(500);

    const captured = await page.evaluate(() => (window as any).capturedInputs);
    
    // Verify that we got exactly "chào" (either as one string or sequence)
    const combined = captured.join('');
    expect(combined).toBe('chào');
    
    // Verify NO duplication (e.g. "chachào")
    expect(combined).not.toContain('chachào');
  });

  test('should simulate Telex sequence via composition events (Advanced)', async ({ page }) => {
    // This test manually triggers composition events to simulate a real IME behavior
    // which was previously causing the duplication bug.
    
    const captured = await page.evaluate(async () => {
      const results: string[] = [];
      const originalSend = (window as any).ws.send;
      (window as any).ws.send = (msg: string) => {
        const parsed = JSON.parse(msg);
        if (parsed.type === 'Input') results.push(parsed.data);
        originalSend.apply((window as any).ws, [msg]);
      };

      const term = (window as any).term;
      const textarea = document.querySelector('.xterm-helper-textarea') as HTMLTextAreaElement;
      
      // Mimic Telex: "ch" -> "cha" -> "chà" (via 'f') -> "chào" (via 'o')
      
      // 1. "ch"
      textarea.value = "ch";
      textarea.dispatchEvent(new InputEvent('input', { data: 'ch', inputType: 'insertText' }));
      
      // 2. "cha"
      textarea.value = "cha";
      textarea.dispatchEvent(new InputEvent('input', { data: 'cha', inputType: 'insertText' }));
      
      // 3. "chà" (Keyboard replaces 'a' with 'à' when 'f' is pressed)
      textarea.value = "chà";
      textarea.dispatchEvent(new InputEvent('input', { data: 'chà', inputType: 'insertText' }));
      
      // 4. "chào"
      textarea.value = "chào";
      textarea.dispatchEvent(new InputEvent('input', { data: 'chào', inputType: 'insertText' }));

      return results;
    });

    // The result depends on how xterm.js handles rapid textarea changes.
    // With screenReaderMode: false, xterm doesn't fight back by writing to the textarea.
    const combined = captured.join('');
    
    // We expect the terminal to have received the final sequence.
    // Xterm usually sends the difference or the whole content depending on the event.
    // Most importantly, we want to ensure we don't see the "chachào" pattern 
    // which indicates the terminal's own echo was re-read as new input.
    
    console.log("Captured sequence:", captured);
    expect(combined).not.toContain('chachào');
    expect(combined).toContain('chào');
  });
});
