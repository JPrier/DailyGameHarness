import { describe, it, expect, beforeEach } from 'vitest';
import { keyFor, saveState, loadState } from '../../src/lib/local-state';

describe('local-state', ()=>{
  beforeEach(()=>localStorage.clear());
  const state:any = {schemaVersion:'daily-game-state.v1',gameId:'g',puzzleId:'p',date:'2026-01-01',status:'in_progress',guessCount:1,maxGuesses:6,currentStage:0,publicState:{}};

  it('round trips valid state', ()=>{
    const k = keyFor('daily-game-runtime.v1','g','p');
    saveState(k,state);
    expect(loadState(k,'g','p','2026-01-01')).toEqual(state);
  });

  it('rejects mismatched game ID', ()=>{
    const k = keyFor('daily-game-runtime.v1','g','p');
    saveState(k,state);
    expect(loadState(k,'other','p','2026-01-01')).toBeNull();
  });

  it('rejects mismatched puzzle ID', ()=>{
    const k = keyFor('daily-game-runtime.v1','g','p');
    saveState(k,state);
    expect(loadState(k,'g','other','2026-01-01')).toBeNull();
  });

  it('rejects unsupported state schema', ()=>{
    const k = keyFor('daily-game-runtime.v1','g','p');
    localStorage.setItem(k, JSON.stringify({...state, schemaVersion:'daily-game-state.v2'}));
    expect(loadState(k,'g','p','2026-01-01')).toBeNull();
  });

  it('rejects private state', ()=>{
    const k = keyFor('daily-game-runtime.v1','g','p');
    localStorage.setItem(k, JSON.stringify({...state, privateState:{answer:'alpha'}}));
    expect(loadState(k,'g','p','2026-01-01')).toBeNull();
  });
});
