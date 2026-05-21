import fs from 'node:fs';
import path from 'node:path';

function parseRegistryFromSource(src) {
  const games = [];
  const lineRe = /"([^"]+)":\s*\{[^\n]*id:\s*"([^"]+)"[^\n]*slug:\s*"([^"]+)"[^\n]*displayName:\s*"([^"]+)"[^\n]*contentManifestUrl:\s*"([^"]+)"[^\n]*dateIndexUrl:\s*"([^"]+)"[^\n]*puzzleBaseUrl:\s*"([^"]+)"[^\n]*dates:\s*\[([^\]]*)\]/g;
  let match;
  while ((match = lineRe.exec(src)) !== null) {
    const [, , id, slug, displayName, contentManifestUrl, dateIndexUrl, puzzleBaseUrl, datesBlob] = match;
    const dates = [...datesBlob.matchAll(/"([^"]+)"/g)].map((x) => x[1]);
    games.push({ id, slug, displayName, contentManifestUrl, dateIndexUrl, puzzleBaseUrl, dates });
  }
  return games;
}

function readPublicJson(url) {
  return JSON.parse(fs.readFileSync(path.join('public', url.replace(/^\//, '')), 'utf8'));
}

function html(title, body, script = '') {
  return `<!doctype html><html><head><meta charset="utf-8"><title>${escapeHtml(title)}</title><link rel="stylesheet" href="/assets/app.css"></head><body>${body}<script src="/assets/app.js"></script>${script}</body></html>`;
}

function escapeHtml(value) {
  return String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
}

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

function gamePage(game, date, manifest, puzzle) {
  const data = JSON.stringify({ game, date, manifest, puzzle }).replaceAll('<', '\\u003c');
  const body = `
    <main data-testid="game-shell">
      <h1>${escapeHtml(game.displayName)}</h1>
      <p data-testid="prompt">${escapeHtml(puzzle.display.initialPrompt)}</p>
      <form id="guess-form" data-testid="guess-form">
        <input id="guess-input" data-testid="guess-input" aria-label="Guess">
        <button id="guess-submit" data-testid="guess-submit" type="submit">Submit</button>
      </form>
      <p data-testid="feedback"></p>
      <p>Guesses: <span data-testid="guess-count">0</span></p>
      <section data-testid="result-modal" hidden></section>
      <button data-testid="share-button" id="share-button" type="button">Share</button>
      <output data-testid="share-output" id="share-output"></output>
    </main>`;
  return html(game.displayName, body, `<script type="application/json" id="game-data">${data}</script><script>window.__dailyGameBoot && window.__dailyGameBoot();</script>`);
}

const regSource = fs.existsSync('src/generated/game-registry.ts')
  ? fs.readFileSync('src/generated/game-registry.ts', 'utf8')
  : '';
const games = parseRegistryFromSource(regSource);

fs.rmSync('dist', { recursive: true, force: true });
fs.mkdirSync('dist/assets', { recursive: true });
write('dist/assets/app.css', 'body{font-family:Georgia,serif;margin:2rem;max-width:52rem}button,input{font:inherit;margin:.25rem;padding:.5rem}.bad{color:#8a1f11}.good{color:#176b2c}');
write(
  'dist/assets/app.js',
  `window.__dailyGameBoot=function(){const el=document.getElementById('game-data');if(!el)return;const data=JSON.parse(el.textContent);const {game,date,manifest,puzzle}=data;const key='daily-game:daily-game-runtime.v1:'+puzzle.gameId+':'+puzzle.puzzleId;const initial={schemaVersion:'daily-game-state.v1',gameId:puzzle.gameId,puzzleId:puzzle.puzzleId,date,status:'in_progress',guessCount:0,maxGuesses:manifest.defaultMaxGuesses||6,currentStage:0,publicState:{history:[]}};let state=initial;try{const saved=JSON.parse(localStorage.getItem(key)||'null');if(saved&&saved.schemaVersion==='daily-game-state.v1'&&saved.gameId===puzzle.gameId&&saved.puzzleId===puzzle.puzzleId&&saved.date===date)state=saved;}catch{}const input=document.getElementById('guess-input');const submit=document.getElementById('guess-submit');const feedback=document.querySelector('[data-testid="feedback"]');const count=document.querySelector('[data-testid="guess-count"]');const result=document.querySelector('[data-testid="result-modal"]');function render(){count.textContent=String(state.guessCount);const done=state.status!=='in_progress';input.disabled=done;submit.disabled=done;if(done){result.hidden=false;result.textContent=state.status==='won'?'Solved':'Game over';}}render();document.getElementById('guess-form').addEventListener('submit',ev=>{ev.preventDefault();if(state.status!=='in_progress')return;const value=input.value.trim().toLowerCase();if(!value){feedback.textContent='invalid';return;}const correct=value===String(puzzle.extension.answer).toLowerCase();const guessCount=state.guessCount+1;state={...state,guessCount,currentStage:guessCount,status:correct?'won':guessCount>=state.maxGuesses?'lost':'in_progress',publicState:{...state.publicState,history:[...(state.publicState.history||[]),value]}};localStorage.setItem(key,JSON.stringify(state));feedback.textContent=correct?'correct':'incorrect';input.value='';render();});document.getElementById('share-button').addEventListener('click',()=>{const text='🎮 '+puzzle.date+' '+state.status+' '+state.guessCount+'/'+state.maxGuesses;document.getElementById('share-output').textContent=text;navigator.clipboard&&navigator.clipboard.writeText&&navigator.clipboard.writeText(text).catch(()=>{});});};`,
);

const homeList = games
  .map((game) => `<li><a href="/games/${game.slug}/">${escapeHtml(game.displayName)}</a></li>`)
  .join('');
write('dist/index.html', html('Daily Games', `<main><h1>Daily Games</h1><ul data-testid="game-list">${homeList}</ul></main>`));

for (const game of games) {
  const manifest = readPublicJson(game.contentManifestUrl);
  const latest = [...game.dates].sort().at(-1);
  for (const date of game.dates) {
    const puzzle = readPublicJson(`${game.puzzleBaseUrl}/${date}.json`);
    write(path.join('dist', 'games', game.slug, date, 'index.html'), gamePage(game, date, manifest, puzzle));
    if (date === latest) {
      write(path.join('dist', 'games', game.slug, 'index.html'), gamePage(game, date, manifest, puzzle));
    }
  }
}

write('dist/404.html', html('Not found', '<main><h1>Game unavailable</h1><p data-testid="not-found">Puzzle unavailable</p></main>'));

if (fs.existsSync('public')) {
  fs.cpSync('public', 'dist', { recursive: true });
}
