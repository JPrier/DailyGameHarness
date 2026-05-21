import { afterEach, describe, it, expect, vi, beforeEach } from 'vitest';
import { cleanup, render, screen, fireEvent } from '@testing-library/svelte';
import GenericFeedback from '../../src/components/GenericFeedback.svelte';
import ShareButton from '../../src/components/ShareButton.svelte';
import { listRegisteredGames } from '../../src/lib/game-registry';
import { loadGameBundle, puzzleUrlFor } from '../../src/lib/game-loader';

describe('registry and loader', () => {
  beforeEach(() => vi.restoreAllMocks());
  afterEach(() => cleanup());

  it('home page registry rendering logic lists configured games', () => {
    expect(listRegisteredGames().map((game) => game.displayName)).toContain('Minimal Text Game');
  });

  it('game loader resolves content manifest and date index URLs', async () => {
    const fetchMock = vi.fn(async (url: string) => ({
      ok: true,
      json: async () => ({ url }),
    }));
    vi.stubGlobal('fetch', fetchMock);
    const bundle = await loadGameBundle('minimal-text-game');
    expect(fetchMock).toHaveBeenCalledWith('/_games/minimal-text-game/content/manifest.json');
    expect(fetchMock).toHaveBeenCalledWith('/_games/minimal-text-game/content/date-index.json');
    expect(bundle.game.slug).toBe('minimal-text-game');
  });

  it('game loader resolves puzzle URL', () => {
    expect(puzzleUrlFor({ puzzleBaseUrl: '/_games/g/content/puzzles' }, '2026-01-01')).toBe(
      '/_games/g/content/puzzles/2026-01-01.json',
    );
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
    const buildShareText = vi.fn(async () => 'share text');
    render(ShareButton, { props: { buildShareText } });
    await fireEvent.click(screen.getByTestId('share-button'));
    expect(buildShareText).toHaveBeenCalled();
    expect(screen.getByTestId('share-output')).toHaveTextContent('share text');
  });
});
