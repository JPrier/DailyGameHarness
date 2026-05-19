import { describe, it, expect } from 'vitest';
import { createRuntime } from '../../../fixtures/games/minimal-text-game/dist/runtime/index.js';

describe('runtime contract fixture', () => {
  it('validatePuzzle accepts valid puzzle', async () => {
    const runtime = await createRuntime();
    const result = await runtime.validatePuzzle({ puzzle: { extension: { answer: 'alpha' } } });
    expect(result.ok).toBe(true);
  });

  it('submitGuess correct/incorrect behavior', async () => {
    const runtime = await createRuntime();
    const puzzle: any = { gameId: 'minimal-text-game', puzzleId: 'p1', date: '2026-01-01', extension: { answer: 'alpha' } };
    const state: any = await runtime.createInitialState({ contentManifest: { defaultMaxGuesses: 6 }, puzzle, date: '2026-01-01' });
    const wrong = await runtime.submitGuess({ puzzle, state, input: { kind: 'text', value: 'beta' } });
    expect(wrong.evaluation.outcome).toBe('incorrect');
    const correct = await runtime.submitGuess({ puzzle, state: wrong.state, input: { kind: 'text', value: 'alpha' } });
    expect(correct.evaluation.outcome).toBe('correct');
  });
});
