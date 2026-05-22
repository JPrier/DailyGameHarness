import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import GameShell from '../../src/components/GameShell.svelte';
import FixtureGameView from '../../../fixtures/games/minimal-text-game/src/GameView.svelte';
import { createRuntime } from '../../../fixtures/games/minimal-text-game/dist/runtime/index.js';

const manifest = {
  schemaVersion: 'daily-game-content-manifest.v1',
  gameId: 'minimal-text-game',
  defaultMaxGuesses: 2,
  inputModes: ['text'],
  puzzleResolver: {
    mode: 'static-pool',
    timezone: 'America/New_York',
    startDate: '2026-01-01',
    poolVersions: [
      {
        version: 'v1',
        startDate: '2026-01-01',
        poolSize: 3,
        pathPattern: 'content/puzzles/v1/puzzle-{index:04}.json',
        selector: { type: 'affine-permutation', a: 2, b: 1 },
        cyclePolicy: 'repeat',
      },
    ],
  },
  archive: { mode: 'rolling-window', days: 30, includeToday: true, allowFutureDates: false, directAccess: 'any-resolvable-date' },
  extension: {},
};
const puzzle = {
  schemaVersion: 'daily-game-puzzle.v1',
  gameId: 'minimal-text-game',
  puzzleId: 'minimal-text-game-2026-01-01',
  date: '2026-01-01',
  seed: 'seed',
  display: { title: 'Puzzle', initialPrompt: 'Guess the answer' },
  extension: { answer: 'alpha' },
};

function response(body: unknown) {
  return { ok: true, json: async () => body };
}

function game(runtimeFactory: any = createRuntime): any {
  return {
    id: 'minimal-text-game',
    slug: 'minimal-text-game',
    displayName: 'Minimal Text Game',
    category: 'fixture',
    routePrefix: '',
    contentManifestUrl: '/manifest.json',
    dateIndexUrl: null,
    puzzleBaseUrl: '/puzzles',
    assetBaseUrl: '/assets',
    runtimeAssetBaseUrl: '/runtime',
    dates: ['2026-01-01'],
    GameView: FixtureGameView,
    createRuntime: runtimeFactory,
  };
}

describe('GameShell.svelte', () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: string) => {
        if (url.includes('manifest')) return response(manifest);
        return response(puzzle);
      }),
    );
  });

  it('renders loading state and loaded puzzle', async () => {
    render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    expect(screen.getByTestId('loading')).toBeInTheDocument();
    expect(await screen.findByTestId('game-shell')).toBeInTheDocument();
    expect(screen.getByTestId('prompt')).toHaveTextContent('Guess the answer');
  });

  it('submits text input, displays feedback, and advances public state', async () => {
    render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    await screen.findByTestId('game-shell');
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'beta' } });
    await fireEvent.click(screen.getByTestId('submit'));
    expect(await screen.findByTestId('feedback')).toHaveTextContent('incorrect');
    expect(screen.getByTestId('guess-count')).toHaveTextContent('1');
  });

  it('displays win result and disables input after completion', async () => {
    render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    await screen.findByTestId('game-shell');
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'alpha' } });
    await fireEvent.click(screen.getByTestId('submit'));
    expect(await screen.findByTestId('result-modal')).toHaveTextContent('Solved');
    expect(screen.getByTestId('guess')).toBeDisabled();
  });

  it('displays loss result', async () => {
    render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    await screen.findByTestId('game-shell');
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'beta' } });
    await fireEvent.click(screen.getByTestId('submit'));
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'gamma' } });
    await fireEvent.click(screen.getByTestId('submit'));
    expect(await screen.findByTestId('result-modal')).toHaveTextContent('Game over');
  });

  it('restores progress after simulated reload', async () => {
    const first = render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    await screen.findByTestId('game-shell');
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'beta' } });
    await fireEvent.click(screen.getByTestId('submit'));
    first.unmount();
    render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    expect(await screen.findByTestId('guess-count')).toHaveTextContent('1');
  });

  it('displays friendly error when runtime validation fails', async () => {
    const badRuntime = async () => ({
      ...(await createRuntime()),
      validatePuzzle: async () => ({ ok: false as const, errors: [{ code: 'bad', message: 'Puzzle invalid' }], warnings: [] }),
    });
    render(GameShell, { props: { game: game(badRuntime), date: '2026-01-01' } });
    expect(await screen.findByTestId('error-panel')).toHaveTextContent('Puzzle invalid');
  });

  it('does not mark a guess correct unless runtime returns correct', async () => {
    const runtime = await createRuntime();
    const conservativeRuntime = async () => ({
      ...runtime,
      submitGuess: async (args: any) => {
        const result = await runtime.submitGuess(args);
        return { ...result, evaluation: { ...result.evaluation, outcome: 'incorrect' as const } };
      },
    });
    render(GameShell, { props: { game: game(conservativeRuntime), date: '2026-01-01' } });
    await screen.findByTestId('game-shell');
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'alpha' } });
    await fireEvent.click(screen.getByTestId('submit'));
    await waitFor(() => expect(screen.getByTestId('feedback')).toHaveTextContent('incorrect'));
  });

  it('renders archive list according to game config', async () => {
    render(GameShell, { props: { game: game(), date: '2026-01-01' } });
    await screen.findByTestId('game-shell');
    expect(screen.getAllByTestId('archive-date')).toHaveLength(30);
  });
});

describe('fixture game view', () => {
  it('receives props, renders prompt, calls submitInput, and does not mutate state directly', async () => {
    const submitInput = vi.fn();
    const state: any = { status: 'in_progress', publicState: { history: [] } };
    render(FixtureGameView, {
      props: { puzzle, state, latestEvaluation: null, submitInput },
    });
    await fireEvent.input(screen.getByTestId('guess'), { target: { value: 'alpha' } });
    await fireEvent.click(screen.getByTestId('submit'));
    expect(screen.getByTestId('prompt')).toHaveTextContent('Guess the answer');
    expect(submitInput).toHaveBeenCalledWith({ kind: 'text', value: 'alpha' });
    expect(state.publicState.history).toEqual([]);
  });
});
