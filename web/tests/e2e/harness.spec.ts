import { test, expect } from '@playwright/test';

test('home page lists the fixture game', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Daily Games' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Minimal Text Game' })).toBeVisible();
});

test('fixture game route loads without console errors', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => message.type() === 'error' && errors.push(message.text()));
  await page.goto('/games/minimal-text-game/');
  await expect(page.getByTestId('prompt')).toHaveText('Guess');
  expect(errors).toEqual([]);
});

test('fixture date query route loads without console errors', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => message.type() === 'error' && errors.push(message.text()));
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  await expect(page.getByTestId('prompt')).toHaveText('Guess');
  expect(errors).toEqual([]);
});

test('player can win the fixture puzzle', async ({ page }) => {
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  await page.getByTestId('guess-input').fill('alpha');
  await page.getByTestId('guess-submit').click();
  await expect(page.getByTestId('feedback')).toHaveText('correct');
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
});

test('player can lose the fixture puzzle', async ({ page }) => {
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  for (const guess of ['one', 'two', 'three', 'four', 'five', 'six']) {
    await page.getByTestId('guess-input').fill(guess);
    await page.getByTestId('guess-submit').click();
  }
  await expect(page.getByTestId('result-modal')).toContainText('Game over');
});

test('refresh after one wrong guess preserves progress', async ({ page }) => {
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  await page.getByTestId('guess-input').fill('wrong');
  await page.getByTestId('guess-submit').click();
  await expect(page.getByTestId('guess-count')).toHaveText('1');
  await page.reload();
  await expect(page.getByTestId('guess-count')).toHaveText('1');
});

test('share button produces non-empty text', async ({ page }) => {
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  await page.getByTestId('share-button').click();
  await expect(page.getByTestId('share-output')).not.toHaveText('');
});

test('invalid game route shows friendly not-found UI', async ({ page }) => {
  await page.goto('/games/not-a-game/');
  await expect(page.getByTestId('not-found')).toContainText('Puzzle unavailable');
});

test('missing puzzle date shows friendly puzzle-not-found UI', async ({ page }) => {
  await page.goto('/games/minimal-text-game/?date=1999-01-01');
  await expect(page.getByTestId('not-found')).toContainText('Puzzle unavailable');
});

test('static-pool load fetches one puzzle and no date index', async ({ page }) => {
  const puzzleRequests: string[] = [];
  const dateIndexRequests: string[] = [];
  page.on('request', (request) => {
    const url = request.url();
    if (url.includes('/content/puzzles/') && url.endsWith('.json')) puzzleRequests.push(url);
    if (url.includes('/content/date-index.json')) dateIndexRequests.push(url);
  });
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  await expect(page.getByTestId('game-shell')).toBeVisible();
  expect(puzzleRequests).toHaveLength(1);
  expect(dateIndexRequests).toHaveLength(0);
});

test('two games expose different archive windows', async ({ page }) => {
  await page.goto('/games/minimal-text-game/?date=2026-05-22');
  await expect(page.getByTestId('archive-date')).toHaveCount(30);
  await page.goto('/games/second-minimal-game/?date=2026-05-22');
  await expect(page.getByTestId('archive-date')).toHaveCount(7);
});

test('built site works when served by a static file server', async ({ page }) => {
  const response = await page.goto('/games/minimal-text-game/?date=2026-05-22');
  expect(response?.ok()).toBe(true);
  await expect(page.getByTestId('game-shell')).toBeVisible();
});
