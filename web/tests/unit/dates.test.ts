import { describe, it, expect } from 'vitest';
import { explicitDate, pickDate } from '../../src/lib/dates';

describe('dates', () => {
  it('date-less route picks today when available', () => {
    expect(pickDate(['2026-01-01', '2026-01-02'], '2026-01-02')).toBe('2026-01-02');
  });

  it('date-less route picks stable latest date when today is unavailable', () => {
    expect(pickDate(['2026-01-01'], '2026-01-02')).toBe('2026-01-01');
  });

  it('explicit date route uses the requested date', () => {
    expect(explicitDate('2026-01-01')).toBe('2026-01-01');
  });
});
