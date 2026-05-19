import { test, expect } from '@playwright/test';

test('home page loads', async ({ page }) => {
  await page.goto('file://' + process.cwd() + '/dist/index.html');
  await expect(page.locator('h1')).toHaveText('Daily Games');
});
