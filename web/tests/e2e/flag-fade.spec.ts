import { test, expect, type Locator, type Page } from '@playwright/test';

const basePath = process.env.E2E_BASE_PATH ?? '';
const appUrl = (path: string) => `${basePath}${path}`;
const fixtureDate = '2026-05-22';

async function openFlagFade(page: Page) {
  await page.goto(appUrl(`/games/flag-fade/?date=${fixtureDate}`));
  const game = page.getByTestId('game-root');
  await expect(game).toBeVisible();
  return game;
}

async function submitGuess(game: Locator, guess: string) {
  await game.getByTestId('guess-input').fill(guess);
  await game.getByTestId('submit-guess').click();
}

test('flag fade route renders without answer leakage', async ({ page, request }) => {
  const errors: string[] = [];
  page.on('console', (message) => message.type() === 'error' && errors.push(message.text()));
  const game = await openFlagFade(page);

  await expect(game.getByTestId('puzzle-title')).toContainText('Flag Fade');
  await expect(game.getByTestId('initial-prompt')).toContainText('Guess the country');
  await expect(game.getByTestId('clue-stage')).toHaveText('0');
  await expect(game.getByTestId('flag-stage-label')).toHaveText('palette');
  await expect(game.getByTestId('flag-image')).toBeVisible();
  await expect(game.getByTestId('guess-input')).toBeVisible();
  await expect(game.getByTestId('guess-count')).toHaveText('0');
  await expect(game.getByTestId('answer-reveal')).toHaveCount(0);
  await expect(game).not.toContainText(/Japan|Nippon|Nihon|JPN/);

  const assetSrc = await game.getByTestId('flag-image').getAttribute('src');
  expect(assetSrc).toContain('/_games/flag-fade/content/assets/');
  expect(assetSrc).toContain('stage-0.svg');
  const assetResponse = await request.get(new URL(assetSrc!, page.url()).href);
  expect(assetResponse.ok()).toBe(true);
  const svg = await assetResponse.text();
  expect(svg).toContain('<svg');
  expect(svg).not.toMatch(/<text[\s>]/i);
  expect(svg).not.toMatch(/Japan|Nippon|Nihon|JPN/i);
  expect(errors).toEqual([]);
});

test('flag fade rejects blank and unknown countries without consuming guesses', async ({ page }) => {
  const game = await openFlagFade(page);

  await game.getByTestId('submit-guess').click();
  await expect(game.getByTestId('status-banner')).toContainText('Enter a country name');
  await expect(game.getByTestId('guess-count')).toHaveText('0');
  await expect(game.getByTestId('guess-history')).toBeEmpty();
  await expect(game.getByTestId('flag-stage-label')).toHaveText('palette');

  await submitGuess(game, 'Atlantis');
  await expect(game.getByTestId('status-banner')).toContainText('Unknown country');
  await expect(game.getByTestId('guess-count')).toHaveText('0');
  await expect(game.getByTestId('guess-history')).toBeEmpty();
  await expect(game.getByTestId('flag-stage-label')).toHaveText('palette');
});

test('flag fade wrong country returns color, continent, design, and layout feedback', async ({ page }) => {
  const game = await openFlagFade(page);

  await submitGuess(game, 'Canada');
  await expect(game.getByTestId('guess-count')).toHaveText('1');
  await expect(game.getByTestId('status-banner')).toContainText('Incorrect');
  await expect(game.getByTestId('guess-history')).toContainText('Canada');
  await expect(game.getByTestId('feedback-row-continent').last()).toBeVisible();
  await expect(game.getByTestId('feedback-row-dominantColors').last()).toContainText(/Exact colors: yes|red|white/);
  await expect(game.getByTestId('feedback-row-emblem').last()).toBeVisible();
  await expect(game.getByTestId('feedback-row-stripeOrientation').last()).toBeVisible();
  await expect(game.getByTestId('feedback-row-aspectRatio').last()).toBeVisible();
  await expect(game.getByTestId('flag-color-feedback')).toContainText(/yes|red|white/);
  await expect(game.getByTestId('flag-continent-feedback')).toContainText(/yes|no/);
  await expect(game.getByTestId('flag-design-feedback')).toContainText(/yes|no/);
});

test('flag fade stages reveal in exact order after valid wrong guesses', async ({ page }) => {
  const game = await openFlagFade(page);
  await expect(game.getByTestId('flag-stage-label')).toHaveText('palette');

  for (const [guess, stage, label] of [
    ['Canada', '1', 'pixelated'],
    ['Bangladesh', '2', 'blurred'],
    ['Indonesia', '3', 'shape and emblem'],
    ['South Korea', '4', 'near-full'],
    ['Brazil', '5', 'full'],
  ] as const) {
    await submitGuess(game, guess);
    await expect(game.getByTestId('clue-stage')).toHaveText(stage);
    await expect(game.getByTestId('flag-stage-label')).toHaveText(label);
    await expect(game.getByTestId('flag-image')).toHaveAttribute('src', new RegExp(`stage-${stage}\\.svg`));
  }
});

test('flag fade accepts canonical and ISO alias answers', async ({ page }) => {
  let game = await openFlagFade(page);
  await submitGuess(game, 'Japan');
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
  await expect(game.getByTestId('answer-reveal')).toContainText('Japan');
  await expect(game.getByTestId('flag-stage-label')).toHaveText('full');
  await expect(game.getByTestId('guess-input')).toBeDisabled();
  await expect(page.getByTestId('share-button')).toBeEnabled();

  await page.evaluate(() => localStorage.clear());
  game = await openFlagFade(page);
  await submitGuess(game, 'JP');
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
  await expect(game.getByTestId('answer-reveal')).toContainText('Japan');
  await expect(game.getByTestId('guess-input')).toBeDisabled();
});

test('flag fade handles similar flag near misses without marking them correct', async ({ page }) => {
  const game = await openFlagFade(page);

  await submitGuess(game, 'Bangladesh');
  await expect(game.getByTestId('guess-count')).toHaveText('1');
  await expect(game.getByTestId('status-banner')).toContainText('Incorrect');
  await expect(game.getByTestId('feedback-row-similar').last()).toContainText('yes');
  await expect(game.getByTestId('flag-similar-feedback')).toContainText('yes');
  await expect(game.getByTestId('answer-reveal')).toHaveCount(0);
});

test('flag fade loses after max wrong guesses and reveals the answer', async ({ page }) => {
  const game = await openFlagFade(page);
  for (const guess of ['Canada', 'Bangladesh', 'Indonesia', 'South Korea', 'Brazil', 'Palau']) {
    await submitGuess(game, guess);
  }

  await expect(page.getByTestId('result-modal')).toContainText('Game over');
  await expect(game.getByTestId('guess-count')).toHaveText('6');
  await expect(game.getByTestId('flag-stage-label')).toHaveText('full');
  await expect(game.getByTestId('answer-reveal')).toContainText('Japan');
  await expect(game.getByTestId('guess-input')).toBeDisabled();
});

test('flag fade persists history and clue stage across refresh', async ({ page }) => {
  const game = await openFlagFade(page);
  await submitGuess(game, 'Canada');
  await expect(game.getByTestId('guess-count')).toHaveText('1');
  await expect(game.getByTestId('flag-stage-label')).toHaveText('pixelated');

  await page.reload();
  const reloadedGame = page.getByTestId('game-root');
  await expect(reloadedGame.getByTestId('guess-count')).toHaveText('1');
  await expect(reloadedGame.getByTestId('guess-history')).toContainText('Canada');
  await expect(reloadedGame.getByTestId('flag-stage-label')).toHaveText('pixelated');
  await expect(reloadedGame.getByTestId('feedback-row-dominantColors').last()).toBeVisible();
});

test('flag fade share text includes result metadata and excludes the answer', async ({ page }) => {
  const game = await openFlagFade(page);
  await submitGuess(game, 'JP');

  await page.getByTestId('share-button').click();
  const output = page.getByTestId('share-output');
  await expect(output).toContainText('Flag Fade');
  await expect(output).toContainText(fixtureDate);
  await expect(output).toContainText('Result: 1/6');
  await expect(output).toContainText(`http://127.0.0.1:4173${basePath}/games/flag-fade/`);
  await expect(output).not.toContainText(/Japan|Nippon|Nihon|JPN/);
  await expect(page.getByTestId('share-link')).toHaveAttribute('href', `http://127.0.0.1:4173${basePath}/games/flag-fade/`);
});

test('flag fade archive route allows current-window dates and rejects old dates', async ({ page }) => {
  await page.goto(appUrl(`/games/flag-fade/?date=${fixtureDate}`));
  await expect(page.getByTestId('game-root')).toBeVisible();

  await page.goto(appUrl('/games/flag-fade/?date=1999-01-01'));
  await expect(page.getByTestId('not-found')).toContainText('Puzzle unavailable');
});

test('flag fade static-pool load fetches one puzzle, one manifest, and staged assets only', async ({ page }) => {
  const manifestRequests: string[] = [];
  const puzzleRequests: string[] = [];
  const assetRequests: string[] = [];
  page.on('request', (request) => {
    const url = request.url();
    if (url.includes('/_games/flag-fade/content/manifest.json')) manifestRequests.push(url);
    if (url.includes('/_games/flag-fade/content/puzzles/') && url.endsWith('.json')) puzzleRequests.push(url);
    if (url.includes('/_games/flag-fade/content/assets/') && url.endsWith('.svg')) assetRequests.push(url);
  });

  const game = await openFlagFade(page);
  await expect(game.getByTestId('flag-image')).toBeVisible();
  expect(manifestRequests).toHaveLength(1);
  expect(puzzleRequests).toHaveLength(1);
  expect(assetRequests).toHaveLength(1);

  await submitGuess(game, 'Canada');
  await expect(game.getByTestId('flag-stage-label')).toHaveText('pixelated');
  expect(puzzleRequests).toHaveLength(1);
  expect(assetRequests.length).toBeLessThanOrEqual(2);
});

test('flag fade controls expose accessible names and keyboard completion', async ({ page }) => {
  const game = await openFlagFade(page);
  await expect(game.getByLabel('Country guess')).toBeVisible();
  await game.getByLabel('Country guess').fill('JP');
  await game.getByLabel('Country guess').press('Enter');
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
  await expect(game.getByRole('button', { name: 'Guess country' })).toBeDisabled();
  await expect(game.getByLabel('Country guess')).toBeDisabled();
});
