import { test, expect } from '../fixtures';

test.describe('Authentication UI', () => {
  test('should show login page when not authenticated', async ({ page, server }) => {
    // Clear cookies to simulate unauthenticated state
    await page.context().clearCookies();
    
    await page.goto('/');
    
    // We expect to be redirected to login.html by the frontend logic
    await page.waitForURL('**/login.html');
    
    // Check for login elements
    await expect(page.locator('.login-box h2')).toContainText('BTerminal');
    await expect(page.locator('button[type="submit"]')).toBeVisible();
    await expect(page.locator('#username')).toBeVisible();
    await expect(page.locator('#password')).toBeVisible();
  });

  test('should allow login with valid credentials', async ({ page, server }) => {
    await page.context().clearCookies();
    await page.goto('/login.html');

    await page.fill('#username', 'admin');
    await page.fill('#password', 'admin');
    await page.click('button[type="submit"]');

    // Should redirect to dashboard
    await page.waitForURL(url => !url.href.includes('login.html'));
    await expect(page.locator('h1')).toContainText('BTerminal');
    await expect(page.locator('button:has-text("Create Session")')).toBeVisible();
  });

  test('should show error on invalid credentials', async ({ page }) => {
    await page.context().clearCookies();
    await page.goto('/login.html');

    await page.fill('#username', 'wrong');
    await page.fill('#password', 'wrong');
    await page.click('button[type="submit"]');

    await expect(page.locator('#errorMessage')).toBeVisible();
    await expect(page.locator('#errorMessage')).toContainText('Invalid username or password');
  });

  test('should persist authentication after reload', async ({ page }) => {
    // The auto-login fixture already logged us in
    await page.goto('/');
    await expect(page.locator('button:has-text("Create Session")')).toBeVisible();

    await page.reload();
    await expect(page.locator('button:has-text("Create Session")')).toBeVisible();
  });
});