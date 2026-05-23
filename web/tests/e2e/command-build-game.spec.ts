import { test, expect } from '@playwright/test';

const basePath = process.env.E2E_BASE_PATH ?? '';
const appUrl = (path: string) => `${basePath}${path}`;

test('command-mode fixture is built and playable through the harness route', async ({ page }) => {
  await page.goto(appUrl('/games/command-build-game/?date=2026-05-22'));
  await expect(page.getByTestId('game-shell')).toBeVisible();
  const game = page.getByTestId('command-build-game-root');
  await expect(game.getByTestId('prompt')).toHaveText('Generated guess');
  await game.getByTestId('guess-input').fill('command');
  await game.getByTestId('guess-submit').click();
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
});

test('command-mode static-pool load fetches one generated puzzle', async ({ page }) => {
  const puzzleRequests: string[] = [];
  page.on('request', (request) => {
    const url = request.url();
    if (url.includes('/_games/command-build-game/content/puzzles/') && url.endsWith('.json')) {
      puzzleRequests.push(url);
    }
  });
  await page.goto(appUrl('/games/command-build-game/?date=2026-05-22'));
  await expect(page.getByTestId('game-shell')).toBeVisible();
  expect(puzzleRequests).toHaveLength(1);
});
