import { test, expect } from '../fixtures';

test.describe('Vietnamese IME Support', () => {
  
  test.describe('Mobile', () => {
    test.beforeEach(async ({ page }) => {
      // Emulate mobile device
      await page.setViewportSize({ width: 375, height: 667 });
      
      const SESSION_NAME = `ime-mobile-test-${Date.now()}`;
      await page.goto('/');
      await page.fill('#new-session-name', SESSION_NAME);
      await page.click('button:has-text("Create Session")');
      await page.waitForSelector('#terminal-view', { state: 'visible' });
      
      // Ensure WebSocket is ready
      await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);
    });

    test('should correctly handle Vietnamese character input without duplication', async ({ page }) => {
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

      // Simulate typing "chào" via playwright keyboard (high level)
      await page.keyboard.type('chào');
      
      await page.waitForTimeout(500);

      const captured = await page.evaluate(() => (window as any).capturedInputs);
      const combined = captured.join('');
      
      expect(combined).toBe('chào');
    });
  });

  test.describe('Desktop', () => {
    test.beforeEach(async ({ page }) => {
      // Emulate desktop device
      await page.setViewportSize({ width: 1280, height: 800 });
      
      const SESSION_NAME = `ime-desktop-test-${Date.now()}`;
      await page.goto('/');
      await page.fill('#new-session-name', SESSION_NAME);
      await page.click('button:has-text("Create Session")');
      await page.waitForSelector('#terminal-view', { state: 'visible' });
      
      await page.waitForFunction(() => (window as any).ws && (window as any).ws.readyState === 1);
    });

    test('should simulate Telex sequence via composition events', async ({ page }) => {
      const captured = await page.evaluate(async () => {
        const results: string[] = [];
        const originalSend = (window as any).ws.send;
        (window as any).ws.send = (msg: string) => {
          const parsed = JSON.parse(msg);
          if (parsed.type === 'Input') results.push(parsed.data);
          originalSend.apply((window as any).ws, [msg]);
        };

        const term = (window as any).term;
        term.focus();
        const textarea = document.querySelector('.xterm-helper-textarea') as HTMLTextAreaElement;
        
        // Mimic Telex: "ch" -> "cha" -> "chà" -> "chào"
        // Proper composition sequence:
        
        // Start composition
        textarea.dispatchEvent(new CompositionEvent('compositionstart', { data: '' }));
        
        // "c"
        textarea.value = "c";
        textarea.dispatchEvent(new CompositionEvent('compositionupdate', { data: 'c' }));
        textarea.dispatchEvent(new InputEvent('input', { data: 'c', inputType: 'insertCompositionText', isComposing: true }));
        await new Promise(r => setTimeout(r, 10));

        // "ch"
        textarea.value = "ch";
        textarea.dispatchEvent(new CompositionEvent('compositionupdate', { data: 'ch' }));
        textarea.dispatchEvent(new InputEvent('input', { data: 'ch', inputType: 'insertCompositionText', isComposing: true }));
        await new Promise(r => setTimeout(r, 10));
        
        // "cha"
        textarea.value = "cha";
        textarea.dispatchEvent(new CompositionEvent('compositionupdate', { data: 'cha' }));
        textarea.dispatchEvent(new InputEvent('input', { data: 'cha', inputType: 'insertCompositionText', isComposing: true }));
        await new Promise(r => setTimeout(r, 10));
        
        // "chà"
        textarea.value = "chà";
        textarea.dispatchEvent(new CompositionEvent('compositionupdate', { data: 'chà' }));
        textarea.dispatchEvent(new InputEvent('input', { data: 'chà', inputType: 'insertCompositionText', isComposing: true }));
        await new Promise(r => setTimeout(r, 10));
        
        // "chào" - End composition
        textarea.value = "chào";
        textarea.dispatchEvent(new CompositionEvent('compositionupdate', { data: 'chào' }));
        textarea.dispatchEvent(new InputEvent('input', { data: 'chào', inputType: 'insertCompositionText', isComposing: true }));
        textarea.dispatchEvent(new CompositionEvent('compositionend', { data: 'chào' }));
        textarea.dispatchEvent(new InputEvent('input', { data: 'chào', inputType: 'insertFromComposition', isComposing: false }));

        await new Promise(r => setTimeout(r, 10));

        return results;
      });

      const combined = captured.join('');
      console.log("Captured sequence:", captured);
      
      // With proper composition handling, we expect clean output.
      // Ideally "chào" or sequence resulting in "chào".
      // Definitely NOT "chchachàchào".
      
      expect(combined.length).toBeLessThan(10); 
      expect(combined).toMatch(/chào$/);
    });

    test('should correctly handle "hiee" -> "hiê" Telex sequence', async ({ page }) => {
      const captured = await page.evaluate(async () => {
        const results: string[] = [];
        const originalSend = (window as any).ws.send;
        (window as any).ws.send = (msg: string) => {
          const parsed = JSON.parse(msg);
          if (parsed.type === 'Input') results.push(parsed.data);
          originalSend.apply((window as any).ws, [msg]);
        };

        const term = (window as any).term;
        term.focus();
        const textarea = document.querySelector('.xterm-helper-textarea') as HTMLTextAreaElement;
        
        const sendComposition = async (data: string, isEnd: boolean = false) => {
          textarea.value = data;
          textarea.dispatchEvent(new CompositionEvent('compositionupdate', { data }));
          textarea.dispatchEvent(new InputEvent('input', { 
              data, 
              inputType: isEnd ? 'insertFromComposition' : 'insertCompositionText', 
              isComposing: !isEnd 
          }));
          if (isEnd) {
              textarea.dispatchEvent(new CompositionEvent('compositionend', { data }));
          }
          await new Promise(r => setTimeout(r, 20));
        };

        // "hiee" -> "hiê"
        textarea.dispatchEvent(new CompositionEvent('compositionstart', { data: '' }));
        await sendComposition('h');
        await sendComposition('hi');
        await sendComposition('hie');
        await sendComposition('hiê', true);

        return results;
      });

      const combined = captured.join('');
      console.log("Captured sequence for hiee:", captured);
      expect(combined).toBe('hiê');
    });
  });
});