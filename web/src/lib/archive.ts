import type { ArchiveConfig } from './types';

export function todayInTimezone(timezone: string, now = new Date()): string {
  return new Intl.DateTimeFormat('en-CA', {
    timeZone: timezone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(now);
}

function addDays(date: string, offset: number): string {
  const value = new Date(`${date}T00:00:00Z`);
  value.setUTCDate(value.getUTCDate() + offset);
  return value.toISOString().slice(0, 10);
}

export function archiveDates(config: ArchiveConfig, today: string): string[] {
  if (config.mode === 'disabled') return [];
  if (config.mode === 'fixed-list') return config.dates;
  if (config.mode !== 'rolling-window') return [];
  const end = config.includeToday ? today : addDays(today, -1);
  return Array.from({ length: config.days }, (_, index) => addDays(end, -index));
}

export function isDateAllowed(config: ArchiveConfig, today: string, date: string): boolean {
  const allowFutureDates =
    config.mode === 'rolling-window' ? config.allowFutureDates : config.mode !== 'disabled' && !!config.allowFutureDates;
  if (!allowFutureDates && date > today) return false;
  const directAccess = 'directAccess' in config ? config.directAccess ?? 'within-archive-window' : 'within-archive-window';
  if (directAccess === 'disabled') return false;
  if (directAccess === 'any-resolvable-date') return true;
  return archiveDates(config, today).includes(date);
}
