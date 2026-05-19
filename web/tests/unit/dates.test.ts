import { describe, it, expect } from 'vitest';
import { pickDate } from '../../src/lib/dates';
it('pick stable date', ()=>{ expect(pickDate(['2026-01-01'],'2026-01-02')).toBe('2026-01-01');});
