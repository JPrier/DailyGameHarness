import type { GameState } from './types';
export const keyFor = (contractVersion:string, gameId:string, puzzleId:string)=>`daily-game:${contractVersion}:${gameId}:${puzzleId}`;
export function saveState(k:string,s:GameState){ localStorage.setItem(k, JSON.stringify(s)); }
export function loadState(k:string, gameId:string, puzzleId:string, date:string): GameState | null { const raw=localStorage.getItem(k); if(!raw) return null; try{ const s=JSON.parse(raw); if(s.schemaVersion!=='daily-game-state.v1') return null; if(s.gameId!==gameId||s.puzzleId!==puzzleId||s.date!==date) return null; return s;}catch{return null;} }
