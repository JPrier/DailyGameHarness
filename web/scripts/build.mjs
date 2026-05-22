import fs from 'node:fs';
import path from 'node:path';

function parseRegistryFromSource(src) {
  const games = [];
  const entryRe = /"([^"]+)":\s*\{([^\n]+)\}/g;
  let match;
  while ((match = entryRe.exec(src)) !== null) {
    const body = match[2];
    const field = (name) => body.match(new RegExp(`${name}:\\s*"([^"]*)"`))?.[1] ?? '';
    const nullableField = (name) => {
      const stringValue = body.match(new RegExp(`${name}:\\s*"([^"]*)"`))?.[1];
      return stringValue ?? null;
    };
    games.push({
      id: field('id'),
      slug: field('slug'),
      displayName: field('displayName'),
      category: field('category'),
      routePrefix: field('routePrefix'),
      contentManifestUrl: field('contentManifestUrl'),
      dateIndexUrl: nullableField('dateIndexUrl'),
      puzzleBaseUrl: field('puzzleBaseUrl'),
      assetBaseUrl: field('assetBaseUrl'),
      runtimeAssetBaseUrl: field('runtimeAssetBaseUrl'),
      dates: [...body.matchAll(/dates:\s*\[([^\]]*)\]/g)]
        .flatMap((x) => [...x[1].matchAll(/"([^"]+)"/g)].map((y) => y[1])),
    });
  }
  return games;
}

function stripRoutePrefix(url, routePrefix = '') {
  if (routePrefix && url.startsWith(routePrefix + '/')) return url.slice(routePrefix.length);
  return url;
}

function readPublicJson(url, routePrefix = '') {
  const rel = stripRoutePrefix(url, routePrefix).replace(/^\//, '');
  return JSON.parse(fs.readFileSync(path.join('public', rel), 'utf8'));
}

function html(title, body, routePrefix = '', script = '') {
  const prefix = routePrefix || '';
  return `<!doctype html><html><head><meta charset="utf-8"><title>${escapeHtml(title)}</title><link rel="stylesheet" href="${prefix}/assets/app.css"></head><body>${body}<script src="${prefix}/assets/app.js"></script>${script}</body></html>`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

function gamePage(game) {
  const data = JSON.stringify({ game }).replaceAll('<', '\\u003c');
  const body = `
    <main data-testid="game-shell">
      <h1>${escapeHtml(game.displayName)}</h1>
      <p data-testid="loading">Loading puzzle...</p>
      <section id="play-area">
        <nav data-testid="archive-list" id="archive-list"></nav>
        <p data-testid="prompt" id="prompt"></p>
        <form id="guess-form" data-testid="guess-form">
          <input id="guess-input" data-testid="guess-input" aria-label="Guess">
          <button id="guess-submit" data-testid="guess-submit" type="submit">Submit</button>
        </form>
        <p data-testid="feedback" id="feedback"></p>
        <p>Guesses: <span data-testid="guess-count" id="guess-count">0</span></p>
        <section data-testid="result-modal" id="result-modal" hidden></section>
        <button data-testid="share-button" id="share-button" type="button">Share</button>
        <output data-testid="share-output" id="share-output"></output>
      </section>
      <section data-testid="not-found" id="not-found" hidden>Puzzle unavailable</section>
    </main>`;
  return html(game.displayName, body, game.routePrefix, `<script type="application/json" id="game-data">${data}</script><script>window.__dailyGameBoot && window.__dailyGameBoot();</script>`);
}

const appJs = String.raw`
function todayInTimezone(timezone){return new Intl.DateTimeFormat('en-CA',{timeZone:timezone,year:'numeric',month:'2-digit',day:'2-digit'}).format(new Date());}
function daysBetween(a,b){return Math.floor((Date.parse(b+'T00:00:00Z')-Date.parse(a+'T00:00:00Z'))/86400000);}
function formatPattern(pattern,{version='',index=0,date}){return pattern.replaceAll('{version}',version).replaceAll('{date}',date).replaceAll('{index:04}',String(index).padStart(4,'0')).replaceAll('{index}',String(index));}
function resolvePuzzle(manifest,date,dateIndex){const r=manifest.puzzleResolver;if(r.mode==='static-pool'){const versions=[...r.poolVersions].filter(v=>v.startDate<=date).sort((a,b)=>a.startDate.localeCompare(b.startDate));const v=versions.at(-1);if(!v)return null;const offset=daysBetween(v.startDate,date);if(offset<0)return null;if(v.cyclePolicy!=='repeat'&&offset>=v.poolSize)return null;const cycle=v.cyclePolicy==='repeat'?offset%v.poolSize:offset;const index=v.selector.type==='affine-permutation'?((v.selector.a*cycle+v.selector.b)%v.poolSize):cycle%v.poolSize;return {date,path:formatPattern(v.pathPattern,{version:v.version,index,date}),assetPath:v.assetPathPattern?formatPattern(v.assetPathPattern,{version:v.version,index,date}):null,index};}if(r.mode==='dated-files')return{date,path:formatPattern(r.pathPattern,{date})};if(r.mode==='date-index'){const found=(dateIndex?.dates||[]).find(x=>x.date===date);return found?{date,path:found.puzzlePath,assetPath:found.assetsPrefix||null}:null;}return null;}
function archiveDates(archive,today){if(archive.mode==='disabled')return[];if(archive.mode==='fixed-list')return archive.dates||[];if(archive.mode!=='rolling-window')return[];const end=new Date((archive.includeToday?today:addDays(today,-1))+'T00:00:00Z');return Array.from({length:archive.days},(_,i)=>addDays(end.toISOString().slice(0,10),-i));}
function addDays(date,offset){const d=new Date(date+'T00:00:00Z');d.setUTCDate(d.getUTCDate()+offset);return d.toISOString().slice(0,10);}
function dateAllowed(archive,today,date){if(!archive.allowFutureDates&&date>today)return false;const direct=archive.directAccess||'within-archive-window';if(direct==='disabled')return false;if(direct==='any-resolvable-date')return true;return archiveDates(archive,today).includes(date);}
async function fetchJson(url){const res=await fetch(url);if(!res.ok)throw new Error('fetch_failed:'+url);return res.json();}
async function loadListedAssets(game,puzzle,stage){const assets=[];if(stage===0)assets.push(...(puzzle.assets?.initial||[]));for(const entry of puzzle.assets?.stages||[]){if(entry.load==='on-reveal'&&entry.stage===stage)assets.push(...(entry.assets||[]));}await Promise.all(assets.map(rel=>fetch(game.assetBaseUrl+'/'+rel.replace(/^content\/assets\//,''))));}
window.__dailyGameBoot=async function(){const el=document.getElementById('game-data');if(!el)return;const {game}=JSON.parse(el.textContent);const loading=document.getElementById('loading');const unavailable=document.getElementById('not-found');function fail(){if(loading)loading.hidden=true;if(unavailable)unavailable.hidden=false;}try{const manifest=await fetchJson(game.contentManifestUrl);const dateIndex=game.dateIndexUrl?await fetchJson(game.dateIndexUrl):null;const today=todayInTimezone(manifest.puzzleResolver.timezone);const selected=new URL(location.href).searchParams.get('date')||today;if(!/^\d{4}-\d{2}-\d{2}$/.test(selected)||!dateAllowed(manifest.archive,today,selected))return fail();const resolved=resolvePuzzle(manifest,selected,dateIndex);if(!resolved)return fail();const puzzle=await fetchJson(game.puzzleBaseUrl+'/'+resolved.path.replace(/^content\/puzzles\//,''));await loadListedAssets(game,puzzle,0);const key='daily-game:daily-game-runtime.v1:'+puzzle.gameId+':'+puzzle.puzzleId+':'+selected;const initial={schemaVersion:'daily-game-state.v1',gameId:puzzle.gameId,puzzleId:puzzle.puzzleId,date:selected,status:'in_progress',guessCount:0,maxGuesses:manifest.defaultMaxGuesses||6,currentStage:0,publicState:{history:[]}};let state=initial;try{const saved=JSON.parse(localStorage.getItem(key)||'null');if(saved&&saved.schemaVersion==='daily-game-state.v1'&&saved.gameId===puzzle.gameId&&saved.puzzleId===puzzle.puzzleId&&saved.date===selected)state=saved;}catch{}const prompt=document.getElementById('prompt');const input=document.getElementById('guess-input');const submit=document.getElementById('guess-submit');const feedback=document.getElementById('feedback');const count=document.getElementById('guess-count');const result=document.getElementById('result-modal');const archive=document.getElementById('archive-list');prompt.textContent=puzzle.display.initialPrompt;for(const date of archiveDates(manifest.archive,today)){const a=document.createElement('a');a.href=game.routePrefix+'/games/'+game.slug+'/?date='+date;a.textContent=date;a.dataset.testid='archive-date';archive.appendChild(a);}function render(){count.textContent=String(state.guessCount);const done=state.status!=='in_progress';input.disabled=done;submit.disabled=done;if(done){result.hidden=false;result.textContent=state.status==='won'?'Solved':'Game over';}}render();if(loading)loading.hidden=true;document.getElementById('guess-form').addEventListener('submit',async ev=>{ev.preventDefault();if(state.status!=='in_progress')return;const value=input.value.trim().toLowerCase();if(!value){feedback.textContent='invalid';return;}const correct=value===String(puzzle.extension.answer).toLowerCase();const guessCount=state.guessCount+1;state={...state,guessCount,currentStage:guessCount,status:correct?'won':guessCount>=state.maxGuesses?'lost':'in_progress',publicState:{...state.publicState,history:[...(state.publicState.history||[]),value]}};localStorage.setItem(key,JSON.stringify(state));feedback.textContent=correct?'correct':'incorrect';input.value='';await loadListedAssets(game,puzzle,state.currentStage);render();});document.getElementById('share-button').addEventListener('click',()=>{const text='🎮 '+selected+' '+state.status+' '+state.guessCount+'/'+state.maxGuesses;document.getElementById('share-output').textContent=text;navigator.clipboard?.writeText?.(text).catch(()=>{});});}catch(err){console.error(err);fail();}};
`;

const regSource = fs.existsSync('src/generated/game-registry.ts')
  ? fs.readFileSync('src/generated/game-registry.ts', 'utf8')
  : '';
const games = parseRegistryFromSource(regSource);

fs.rmSync('dist', { recursive: true, force: true });
fs.mkdirSync('dist/assets', { recursive: true });
write('dist/assets/app.css', 'body{font-family:Georgia,serif;margin:2rem;max-width:52rem}button,input{font:inherit;margin:.25rem;padding:.5rem}#archive-list{display:flex;flex-wrap:wrap;gap:.4rem;margin:1rem 0}#archive-list a{border:1px solid #ccc;padding:.25rem .4rem}.bad{color:#8a1f11}.good{color:#176b2c}');
write('dist/assets/app.js', appJs);

const routePrefix = games[0]?.routePrefix ?? '';
const homeList = games
  .map((game) => `<li><a href="${game.routePrefix}/games/${game.slug}/">${escapeHtml(game.displayName)}</a></li>`)
  .join('');
write('dist/index.html', html('Daily Games', `<main><h1>Daily Games</h1><ul data-testid="game-list">${homeList}</ul></main>`, routePrefix));

for (const game of games) {
  readPublicJson(game.contentManifestUrl, game.routePrefix);
  write(path.join('dist', 'games', game.slug, 'index.html'), gamePage(game));
}

write('dist/404.html', html('Not found', '<main><h1>Game unavailable</h1><p data-testid="not-found">Puzzle unavailable</p></main>', routePrefix));

if (fs.existsSync('public')) {
  fs.cpSync('public', 'dist', { recursive: true });
}
