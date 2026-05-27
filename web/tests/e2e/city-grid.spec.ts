import { test, expect, type Locator, type Page } from '@playwright/test';

const basePath = process.env.E2E_BASE_PATH ?? '';
const appUrl = (path: string) => `${basePath}${path}`;
const fixtureDate = '2026-05-22';
const fixtureAnswer = 'Helsinki';
const fixtureAlias = 'Helsinki, FI';
const wrongGuesses = ['Paris', 'London', 'Tokyo', 'Rome', 'Singapore', 'Berlin'];

async function openCityGrid(page: Page) {
  await page.goto(appUrl(`/games/city-grid/?date=${fixtureDate}`));
  const game = page.getByTestId('game-root');
  await expect(game).toBeVisible();
  return game;
}

async function submitGuess(game: Locator, guess: string) {
  await game.getByTestId('guess-input').fill(guess);
  await game.getByTestId('submit-guess').click();
}

test('city grid route renders the public game contract without leaking the answer initially', async ({ page, request }) => {
  const errors: string[] = [];
  page.on('console', (message) => message.type() === 'error' && errors.push(message.text()));
  const game = await openCityGrid(page);

  await expect(game.getByTestId('puzzle-title')).toContainText('City Grid');
  await expect(game.getByTestId('initial-prompt')).toContainText('Identify the city');
  await expect(game.getByTestId('clue-stage')).toHaveText('0');
  await expect(game.getByTestId('clue-stage')).toBeHidden();
  await expect(game.getByTestId('city-grid-map')).toBeVisible();
  await expect(game.getByTestId('city-grid-map-stage')).toContainText('Stage 0');
  await expect(game.getByTestId('city-grid-map-legend')).toContainText('Major roads');
  await expect(game.getByTestId('city-grid-map-legend')).toContainText('Water');
  await expect(game.getByTestId('guess-count')).toContainText('0 / 6');
  await expect(page.getByTestId('guess-input')).toHaveCount(1);
  await expect(page.getByTestId('guess-count')).toHaveCount(1);
  await expect(game.getByTestId('answer-reveal')).toHaveCount(0);

  const assetSrc = await game.getByTestId('city-grid-stage-asset').getAttribute('src');
  expect(assetSrc).toContain('/_games/city-grid/content/assets/');
  const assetResponse = await request.get(new URL(assetSrc!, page.url()).href);
  expect(assetResponse.ok()).toBe(true);
  const svg = await assetResponse.text();
  expect(svg).toContain('<path');
  expect(svg).not.toMatch(/<text[\s>]/i);
  expect(svg.toLowerCase()).not.toContain(fixtureAnswer.toLowerCase());
  expect(errors).toEqual([]);
});

test('city grid rejects blank and unknown guesses without consuming attempts', async ({ page }) => {
  const game = await openCityGrid(page);

  await game.getByTestId('submit-guess').click();
  await expect(game.getByTestId('status-banner')).toContainText('Enter a city name');
  await expect(game.getByTestId('guess-count')).toContainText('0 / 6');

  await submitGuess(game, 'Notacityville');
  await expect(game.getByTestId('status-banner')).toContainText('Unknown city');
  await expect(game.getByTestId('guess-count')).toContainText('0 / 6');
});

test('city grid consumes valid wrong guesses and advances staged feedback', async ({ page }) => {
  const game = await openCityGrid(page);
  const firstAsset = await game.getByTestId('city-grid-stage-asset').getAttribute('src');

  await submitGuess(game, wrongGuesses[0]);
  await expect(game.getByTestId('guess-count')).toContainText('1 / 6');
  await expect(game.getByTestId('clue-stage')).toHaveText('1');
  await expect(game.getByTestId('status-banner')).toContainText('Incorrect');
  await expect(game.getByTestId('latest-feedback-proximity')).toBeVisible();
  await expect(game.getByTestId('latest-feedback-distance')).toBeVisible();
  await expect(game.getByTestId('latest-feedback-direction')).toBeVisible();
  await expect(game.getByTestId('latest-feedback-sameCountry')).toBeVisible();
  await expect(game.getByTestId('latest-feedback-population')).toBeVisible();
  await expect(game.getByTestId('city-grid-distance-feedback')).toContainText('mi');
  await expect(game.getByTestId('city-grid-direction-feedback')).toContainText('Go');
  await expect(game.getByTestId('guess-history')).toContainText(wrongGuesses[0]);

  const secondAsset = await game.getByTestId('city-grid-stage-asset').getAttribute('src');
  expect(secondAsset).not.toBe(firstAsset);

  for (const [guess, stage] of [
    [wrongGuesses[1], '2'],
    [wrongGuesses[2], '3'],
    [wrongGuesses[3], '4'],
    [wrongGuesses[4], '5'],
  ] as const) {
    await submitGuess(game, guess);
    await expect(game.getByTestId('clue-stage')).toHaveText(stage);
  }
});

test('city grid accepts canonical and alias answers', async ({ page }) => {
  let game = await openCityGrid(page);
  await submitGuess(game, fixtureAnswer);
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
  await expect(game.getByTestId('answer-reveal')).toContainText(fixtureAnswer);
  await expect(game.getByTestId('guess-input')).toBeDisabled();

  await page.evaluate(() => localStorage.clear());
  game = await openCityGrid(page);
  await submitGuess(game, fixtureAlias);
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
  await expect(game.getByTestId('answer-reveal')).toContainText(fixtureAnswer);
});

test('city grid loses after six wrong valid guesses and reveals the answer', async ({ page }) => {
  const game = await openCityGrid(page);
  for (const guess of wrongGuesses) {
    await submitGuess(game, guess);
  }

  await expect(page.getByTestId('result-modal')).toContainText('Game over');
  await expect(game.getByTestId('guess-count')).toContainText('6 / 6');
  await expect(game.getByTestId('clue-stage')).toHaveText('5');
  await expect(game.getByTestId('answer-reveal')).toContainText(fixtureAnswer);
  await expect(game.getByTestId('guess-input')).toBeDisabled();
});

test('city grid persists progress across refresh', async ({ page }) => {
  const game = await openCityGrid(page);
  await submitGuess(game, wrongGuesses[0]);
  await expect(game.getByTestId('guess-count')).toContainText('1 / 6');
  await page.reload();
  const reloadedGame = page.getByTestId('game-root');
  await expect(reloadedGame.getByTestId('guess-count')).toContainText('1 / 6');
  await expect(reloadedGame.getByTestId('guess-history')).toContainText(wrongGuesses[0]);
});

test('city grid share text includes the full game URL and excludes the answer', async ({ page }) => {
  const game = await openCityGrid(page);
  await submitGuess(game, fixtureAnswer);

  await page.getByTestId('share-button').click();
  const output = page.getByTestId('share-output');
  await expect(output).toContainText(`http://127.0.0.1:4173${basePath}/games/city-grid/`);
  await expect(output).toContainText('City Grid');
  await expect(output).not.toContainText(fixtureAnswer);
  await expect(page.getByTestId('share-link')).toHaveAttribute(
    'href',
    `http://127.0.0.1:4173${basePath}/games/city-grid/`,
  );
});

test('city grid archive policy allows current-window dates and rejects old dates', async ({ page }) => {
  await page.goto(appUrl(`/games/city-grid/?date=${fixtureDate}`));
  await expect(page.getByTestId('game-root')).toBeVisible();

  await page.goto(appUrl('/games/city-grid/?date=1999-01-01'));
  await expect(page.getByTestId('not-found')).toContainText('Puzzle unavailable');
});

test('city grid static-pool load fetches one puzzle, one manifest, and staged assets only', async ({ page }) => {
  const manifestRequests: string[] = [];
  const puzzleRequests: string[] = [];
  const assetRequests: string[] = [];
  page.on('request', (request) => {
    const url = request.url();
    if (url.includes('/_games/city-grid/content/manifest.json')) manifestRequests.push(url);
    if (url.includes('/_games/city-grid/content/puzzles/') && url.endsWith('.json')) puzzleRequests.push(url);
    if (url.includes('/_games/city-grid/content/assets/') && url.endsWith('.svg')) assetRequests.push(url);
  });

  const game = await openCityGrid(page);
  await expect(game.getByTestId('city-grid-stage-asset')).toBeVisible();
  expect(manifestRequests).toHaveLength(1);
  expect(puzzleRequests).toHaveLength(1);
  expect(assetRequests).toHaveLength(1);

  await submitGuess(game, wrongGuesses[0]);
  await expect(game.getByTestId('clue-stage')).toHaveText('1');
  expect(puzzleRequests).toHaveLength(1);
  expect(assetRequests.length).toBeLessThanOrEqual(2);
});

test('city grid core controls expose accessible names and keyboard completion', async ({ page }) => {
  const game = await openCityGrid(page);
  await expect(game.getByLabel('City guess')).toBeVisible();
  await game.getByLabel('City guess').fill(fixtureAnswer);
  await game.getByLabel('City guess').press('Enter');
  await expect(page.getByTestId('result-modal')).toContainText('Solved');
  await expect(game.getByRole('button', { name: 'Guess city' })).toBeDisabled();
});
