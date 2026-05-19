import fs from 'node:fs';
import path from 'node:path';

function parseRegistryFromSource(src) {
  const games = [];
  const lineRe = /"([^"]+)":\s*\{[^\n]*slug:\s*"([^"]+)"[^\n]*displayName:\s*"([^"]+)"[^\n]*dates:\s*\[([^\]]*)\]/g;
  let m;
  while ((m = lineRe.exec(src)) !== null) {
    const [, , slug, displayName, datesBlob] = m;
    const dates = [...datesBlob.matchAll(/"([^"]+)"/g)].map((x) => x[1]);
    games.push({ slug, displayName, dates });
  }
  return games;
}

const regSource = fs.existsSync('src/generated/game-registry.ts')
  ? fs.readFileSync('src/generated/game-registry.ts', 'utf8')
  : '';
const games = parseRegistryFromSource(regSource);

fs.mkdirSync('dist/games', { recursive: true });
fs.writeFileSync('dist/index.html', '<html><body><h1>Daily Games</h1></body></html>');

for (const g of games) {
  fs.mkdirSync(path.join('dist', 'games', g.slug), { recursive: true });
  fs.writeFileSync(path.join('dist', 'games', g.slug, 'index.html'), `<html><body>${g.displayName}</body></html>`);
  for (const d of g.dates) {
    fs.mkdirSync(path.join('dist', 'games', g.slug, d), { recursive: true });
    fs.writeFileSync(path.join('dist', 'games', g.slug, d, 'index.html'), `<html><body>${g.displayName} ${d}</body></html>`);
  }
}

if (fs.existsSync('public')) {
  fs.cpSync('public', 'dist', { recursive: true });
}
