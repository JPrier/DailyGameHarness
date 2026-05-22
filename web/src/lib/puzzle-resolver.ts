import type { ContentManifest, DateIndex, ResolvedPuzzleRef } from './types';

function daysBetween(start: string, end: string): number {
  return Math.floor((Date.parse(`${end}T00:00:00Z`) - Date.parse(`${start}T00:00:00Z`)) / 86_400_000);
}

export function formatPattern(pattern: string, args: { version?: string; index?: number; date: string }): string {
  const index = args.index ?? 0;
  return pattern
    .replaceAll('{version}', args.version ?? '')
    .replaceAll('{date}', args.date)
    .replaceAll('{index:04}', String(index).padStart(4, '0'))
    .replaceAll('{index}', String(index));
}

export function resolvePuzzle(
  manifest: Pick<ContentManifest, 'puzzleResolver'>,
  date: string,
  dateIndex: DateIndex | null = null,
): ResolvedPuzzleRef | null {
  const resolver = manifest.puzzleResolver;
  if (resolver.mode === 'dated-files') {
    return { date, path: formatPattern(resolver.pathPattern, { date }) };
  }
  if (resolver.mode === 'date-index') {
    const entry = dateIndex?.dates.find((item) => item.date === date);
    return entry ? { date, path: entry.puzzlePath, assetPath: entry.assetsPrefix } : null;
  }

  const version = [...resolver.poolVersions]
    .filter((candidate) => candidate.startDate <= date)
    .sort((a, b) => a.startDate.localeCompare(b.startDate))
    .at(-1);
  if (!version) return null;

  const offset = daysBetween(version.startDate, date);
  if (offset < 0) return null;
  if (version.cyclePolicy !== 'repeat' && offset >= version.poolSize) return null;
  const cycleOffset = version.cyclePolicy === 'repeat' ? offset % version.poolSize : offset;
  const index =
    version.selector.type === 'affine-permutation'
      ? (version.selector.a * cycleOffset + version.selector.b) % version.poolSize
      : cycleOffset % version.poolSize;

  return {
    date,
    index,
    path: formatPattern(version.pathPattern, { version: version.version, index, date }),
    assetPath: version.assetPathPattern
      ? formatPattern(version.assetPathPattern, { version: version.version, index, date })
      : undefined,
  };
}
