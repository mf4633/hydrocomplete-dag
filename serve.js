// Minimal static file server for E2E testing — serves www/ with correct MIME types.
// Usage: node serve.js [port]
const http = require('http');
const fs   = require('fs');
const path = require('path');

const port = parseInt(process.argv[2] || '7777', 10);
const root = path.join(__dirname, 'www');

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js':   'application/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.css':  'text/css',
  '.json': 'application/json',
  '.png':  'image/png',
  '.ts':   'application/typescript',
};

http.createServer((req, res) => {
  const url   = req.url.split('?')[0];
  const file  = url === '/' ? '/index.html' : url;
  const full  = path.join(root, file);

  // Security: stay inside root
  if (!full.startsWith(root)) { res.writeHead(403); res.end(); return; }

  fs.readFile(full, (err, data) => {
    if (err) { res.writeHead(404); res.end('Not found'); return; }
    const ext  = path.extname(full).toLowerCase();
    const mime = MIME[ext] || 'application/octet-stream';
    res.writeHead(200, { 'Content-Type': mime });
    res.end(data);
  });
}).listen(port, '127.0.0.1', () => {
  console.log(`HydroComplete DAG test server: http://127.0.0.1:${port}`);
});
