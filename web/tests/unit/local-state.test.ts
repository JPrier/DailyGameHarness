import { describe, it, expect, beforeEach } from 'vitest';
import { keyFor, saveState, loadState } from '../../src/lib/local-state';

describe('local-state', ()=>{
  beforeEach(()=>localStorage.clear());
  it('round trips', ()=>{
    const k = keyFor('daily-game-runtime.v1','g','p');
    const s:any = {schemaVersion:'daily-game-state.v1',gameId:'g',puzzleId:'p',date:'2026-01-01',status:'in_progress',guessCount:1,maxGuesses:6,currentStage:0,publicState:{}};
    saveState(k,s);
    expect(loadState(k,'g','p','2026-01-01')).toEqual(s);
  });
});
