import { describe, it, expect } from 'vitest';
import { generatedGameRegistry } from '../../src/generated/game-registry';
import { createRuntime } from '../../../fixtures/games/minimal-text-game/dist/runtime/index.js';

const manifest = {
  schemaVersion: 'daily-game-content-manifest.v1',
  gameId: 'minimal-text-game',
  defaultMaxGuesses: 6,
  inputModes: ['text'],
  extension: {},
};
const dateIndex = {
  schemaVersion: 'daily-game-date-index.v1',
  gameId: 'minimal-text-game',
  dates: [{ date: '2026-01-01', puzzlePath: 'content/puzzles/2026-01-01.json' }],
};
const packageConfig = { game: { id: 'minimal-text-game' } };
const puzzle: any = {
  schemaVersion: 'daily-game-puzzle.v1',
  gameId: 'minimal-text-game',
  puzzleId: 'p1',
  date: '2026-01-01',
  display: { initialPrompt: 'Guess' },
  extension: { answer: 'alpha' },
};

describe('runtime contract fixture', () => {
  it('runtime adapter can be imported from generated registry', async () => {
    const runtime: any = await generatedGameRegistry['minimal-text-game'].createRuntime();
    expect(runtime.contractVersion).toBe('daily-game-runtime.v1');
  });

  it('validateContent accepts valid fixture content', async () => {
    const runtime: any = await createRuntime();
    await expect(runtime.validateContent({ packageConfig, contentManifest: manifest, dateIndex })).resolves.toMatchObject({ ok: true });
  });

  it('validatePuzzle accepts valid fixture puzzle and rejects invalid puzzle', async () => {
    const runtime: any = await createRuntime();
    await expect(runtime.validatePuzzle({ contentManifest: manifest, puzzle })).resolves.toMatchObject({ ok: true });
    await expect(runtime.validatePuzzle({ contentManifest: manifest, puzzle: { ...puzzle, extension: {} } })).resolves.toMatchObject({ ok: false });
  });

  it('createInitialState returns valid shared state', async () => {
    const runtime: any = await createRuntime();
    const state = await runtime.createInitialState({ contentManifest: manifest, puzzle, date: '2026-01-01' });
    expect(state).toMatchObject({
      schemaVersion: 'daily-game-state.v1',
      gameId: 'minimal-text-game',
      puzzleId: 'p1',
      status: 'in_progress',
      publicState: { history: [] },
    });
    expect(state).not.toHaveProperty('privateState');
  });

  it('submitGuess handles wrong, correct, invalid, and terminal guesses', async () => {
    const runtime: any = await createRuntime();
    const state = await runtime.createInitialState({ contentManifest: manifest, puzzle, date: '2026-01-01' });
    const invalid = await runtime.submitGuess({ contentManifest: manifest, puzzle, state, input: { kind: 'text', value: '' } });
    expect(invalid.evaluation).toMatchObject({ outcome: 'invalid', consumedGuess: false });
    const wrong = await runtime.submitGuess({ contentManifest: manifest, puzzle, state, input: { kind: 'text', value: 'beta' } });
    expect(wrong.evaluation).toMatchObject({ outcome: 'incorrect', consumedGuess: true });
    const correct = await runtime.submitGuess({ contentManifest: manifest, puzzle, state: wrong.state, input: { kind: 'text', value: 'alpha' } });
    expect(correct.evaluation.outcome).toBe('correct');
    const afterDone = await runtime.submitGuess({ contentManifest: manifest, puzzle, state: correct.state, input: { kind: 'text', value: 'alpha' } });
    expect(afterDone.evaluation).toMatchObject({ outcome: 'invalid', consumedGuess: false });
  });

  it('buildShareText returns non-empty spoiler-safe text', async () => {
    const runtime: any = await createRuntime();
    const state = await runtime.createInitialState({ contentManifest: manifest, puzzle, date: '2026-01-01' });
    const share = await runtime.buildShareText({ contentManifest: manifest, puzzle, state });
    expect(share).toContain('2026-01-01');
    expect(share).not.toContain('alpha');
  });

  it('runtime calls are deterministic for same inputs', async () => {
    const runtime: any = await createRuntime();
    const state = await runtime.createInitialState({ contentManifest: manifest, puzzle, date: '2026-01-01' });
    const one = await runtime.submitGuess({ contentManifest: manifest, puzzle, state, input: { kind: 'text', value: 'beta' } });
    const two = await runtime.submitGuess({ contentManifest: manifest, puzzle, state, input: { kind: 'text', value: 'beta' } });
    expect(two).toEqual(one);
  });
});
