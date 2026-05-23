import { afterEach, describe, it, expect, vi, beforeEach } from 'vitest';
import { cleanup, render, screen, fireEvent } from '@testing-library/svelte';
import GenericFeedback from '../../src/components/GenericFeedback.svelte';
import ShareButton from '../../src/components/ShareButton.svelte';
import { listRegisteredGames } from '../../src/lib/game-registry';
import { loadGameBundle, puzzleUrlFor } from '../../src/lib/game-loader';
import { archiveDates } from '../../src/lib/archive';
import { resolvePuzzle } from '../../src/lib/puzzle-resolver';

describe('registry and loader', () => {
  beforeEach(() => vi.restoreAllMocks());
  afterEach(() => cleanup());

  it('home page registry rendering logic lists configured games', () => {
    expect(listRegisteredGames().map((game) => game.displayName)).toContain('Minimal Text Game');
  });

  it('game loader resolves content manifest without requiring a date index', async () => {
    const fetchMock = vi.fn(async (url: string) => ({
      ok: true,
      json: async () => ({ url }),
    }));
    vi.stubGlobal('fetch', fetchMock);
    const bundle = await loadGameBundle('minimal-text-game');
    expect(fetchMock).toHaveBeenCalledWith('/_games/minimal-text-game/content/manifest.json');
    expect(fetchMock).not.toHaveBeenCalledWith('/_games/minimal-text-game/content/date-index.json');
    expect(bundle.game.slug).toBe('minimal-text-game');
  });

  it('game loader resolves puzzle URL', () => {
    expect(puzzleUrlFor({ puzzleBaseUrl: '/_games/g/content/puzzles' }, { date: '2026-01-01', path: 'content/puzzles/v1/puzzle-0001.json' })).toBe(
      '/_games/g/content/puzzles/v1/puzzle-0001.json',
    );
  });

  it('static-pool resolver returns one selected puzzle path', () => {
    const manifest: any = {
      puzzleResolver: {
        mode: 'static-pool',
        timezone: 'America/New_York',
        startDate: '2026-01-01',
        poolVersions: [{ version: 'v1', startDate: '2026-01-01', poolSize: 3, pathPattern: 'content/puzzles/{version}/puzzle-{index:04}.json', selector: { type: 'affine-permutation', a: 2, b: 1 }, cyclePolicy: 'repeat' }],
      },
    };
    expect(resolvePuzzle(manifest, '2026-01-02')?.path).toBe('content/puzzles/v1/puzzle-0000.json');
  });

  it('archive list respects per-game rolling windows', () => {
    expect(archiveDates({ mode: 'rolling-window', days: 30, includeToday: true, allowFutureDates: false }, '2026-05-22')).toHaveLength(30);
    expect(archiveDates({ mode: 'rolling-window', days: 7, includeToday: true, allowFutureDates: false }, '2026-05-22')).toHaveLength(7);
  });
});

describe('generic UI helpers', () => {
  it('generic feedback renderer handles every shared feedback kind', () => {
    const kinds = ['text', 'number', 'direction', 'distance', 'comparison', 'boolean', 'custom'] as const;
    render(GenericFeedback, {
      props: {
        feedback: kinds.map((kind) => ({ key: kind, label: kind, kind, value: kind, severity: 'neutral' })),
      },
    });
    for (const kind of kinds) {
      expect(screen.getByTestId(`feedback-${kind}`)).toBeInTheDocument();
    }
  });

  it('share button calls runtime share function', async () => {
    const buildShareText = vi.fn(async () => 'share text\nhttps://example.test/games/minimal-text-game/');
    render(ShareButton, { props: { buildShareText } });
    await fireEvent.click(screen.getByTestId('share-button'));
    expect(buildShareText).toHaveBeenCalled();
    expect(screen.getByTestId('share-output')).toHaveTextContent('share text');
    expect(screen.getByTestId('share-link')).toHaveAttribute('href', 'https://example.test/games/minimal-text-game/');
  });
});
