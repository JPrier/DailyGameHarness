import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('dist');
const port = 4173;

function contentType(file) {
  if (file.endsWith('.html')) return 'text/html';
  if (file.endsWith('.js')) return 'text/javascript';
  if (file.endsWith('.css')) return 'text/css';
  if (file.endsWith('.json')) return 'application/json';
  return 'application/octet-stream';
}

http
  .createServer((req, res) => {
    const rawUrl = decodeURIComponent(new URL(req.url ?? '/', 'http://127.0.0.1').pathname);
    let file = path.join(root, rawUrl);
    if (rawUrl.endsWith('/')) file = path.join(file, 'index.html');
    if (!path.extname(file)) file = path.join(file, 'index.html');
    if (!file.startsWith(root)) {
      res.writeHead(403);
      res.end('forbidden');
      return;
    }
    if (!fs.existsSync(file)) {
      const notFound = path.join(root, '404.html');
      res.writeHead(404, { 'content-type': 'text/html' });
      res.end(fs.readFileSync(notFound, 'utf8'));
      return;
    }
    res.writeHead(200, { 'content-type': contentType(file) });
    res.end(fs.readFileSync(file));
  })
  .listen(port, '127.0.0.1', () => {
    console.log(`static server listening on ${port}`);
  });
