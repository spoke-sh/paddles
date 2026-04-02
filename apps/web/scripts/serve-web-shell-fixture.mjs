import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const port = Number.parseInt(process.env.PORT || '4173', 10);
const htmlPath = path.resolve(__dirname, '../../../src/infrastructure/web/index.html');
const faviconPath = path.resolve(__dirname, '../public/favicon.svg');

const html = await readFile(htmlPath, 'utf8');
const favicon = await readFile(faviconPath);

const sessionId = 'fixture-session';
const emptyForensics = {
  task_id: sessionId,
  turns: []
};
const emptyManifold = {
  task_id: sessionId,
  turns: []
};
const traceGraph = {
  nodes: [
    { id: 'root-1', kind: 'root', label: 'task root', branch_id: null, sequence: 1 },
    { id: 'turn-1', kind: 'turn', label: 'first turn', branch_id: null, sequence: 2 },
    { id: 'action-1', kind: 'action', label: 'inspect `git status --short`', branch_id: null, sequence: 3 },
    { id: 'evidence-1', kind: 'evidence', label: 'git status output', branch_id: null, sequence: 4 },
    { id: 'forensic-1', kind: 'forensic', label: 'planner request envelope', branch_id: null, sequence: 5 },
    { id: 'signal-1', kind: 'signal', label: 'premise challenge', branch_id: null, sequence: 6 },
  ],
  edges: [
    { from: 'root-1', to: 'turn-1' },
    { from: 'turn-1', to: 'action-1' },
    { from: 'action-1', to: 'evidence-1' },
    { from: 'evidence-1', to: 'forensic-1' },
    { from: 'forensic-1', to: 'signal-1' },
  ],
  branches: []
};

const transcript = {
  task_id: sessionId,
  entries: []
};

function json(res, statusCode, payload) {
  res.writeHead(statusCode, { 'content-type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify(payload));
}

function htmlResponse(res, statusCode, body) {
  res.writeHead(statusCode, { 'content-type': 'text/html; charset=utf-8' });
  res.end(body);
}

function textResponse(res, statusCode, body, contentType = 'text/plain; charset=utf-8') {
  res.writeHead(statusCode, { 'content-type': contentType });
  res.end(body);
}

function openEventStream(req, res) {
  res.writeHead(200, {
    'content-type': 'text/event-stream; charset=utf-8',
    'cache-control': 'no-cache, no-transform',
    connection: 'keep-alive',
  });
  res.write(': fixture\n\n');
  const heartbeat = setInterval(() => {
    res.write(': keep-alive\n\n');
  }, 15_000);
  req.on('close', () => {
    clearInterval(heartbeat);
    res.end();
  });
}

function parseJsonBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', (chunk) => {
      body += chunk;
    });
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch (error) {
        reject(error);
      }
    });
    req.on('error', reject);
  });
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url || '/', `http://127.0.0.1:${port}`);
  const pathname = url.pathname;

  if (req.method === 'GET' && pathname === '/health') {
    return json(res, 200, { status: 'ok' });
  }

  if (req.method === 'GET' && pathname === '/favicon.ico') {
    return textResponse(res, 200, favicon, 'image/svg+xml');
  }

  if (req.method === 'POST' && pathname === '/sessions') {
    return json(res, 200, { session_id: sessionId });
  }

  if (req.method === 'GET' && pathname === `/sessions/${sessionId}/transcript`) {
    return json(res, 200, transcript);
  }

  if (req.method === 'POST' && pathname === `/sessions/${sessionId}/turns`) {
    const payload = await parseJsonBody(req);
    const prompt = String(payload.prompt || '').trim();
    const turnIndex = transcript.entries.length / 2 + 1;
    transcript.entries.push(
      {
        speaker: 'user',
        content: prompt,
        record_id: `user-${turnIndex}`,
      },
      {
        speaker: 'assistant',
        content: `Fixture assistant response for: ${prompt}`,
        record_id: `assistant-${turnIndex}`,
      }
    );
    return json(res, 200, {
      response: `Fixture assistant response for: ${prompt}`,
    });
  }

  if (req.method === 'GET' && pathname === `/sessions/${sessionId}/forensics`) {
    return json(res, 200, emptyForensics);
  }

  if (req.method === 'GET' && pathname === `/sessions/${sessionId}/manifold`) {
    return json(res, 200, emptyManifold);
  }

  if (req.method === 'GET' && pathname === '/trace/graph') {
    return json(res, 200, traceGraph);
  }

  if (
    req.method === 'GET' &&
    (
      pathname === `/sessions/${sessionId}/events` ||
      pathname === `/sessions/${sessionId}/transcript/events` ||
      pathname === `/sessions/${sessionId}/forensics/events` ||
      pathname === `/sessions/${sessionId}/manifold/events` ||
      pathname === '/events'
    )
  ) {
    return openEventStream(req, res);
  }

  if (req.method === 'GET' && (pathname === '/' || pathname === '/manifold' || pathname === '/transit')) {
    return htmlResponse(res, 200, html);
  }

  return json(res, 404, { error: 'not_found', path: pathname });
});

server.listen(port, '127.0.0.1', () => {
  process.stdout.write(`fixture server listening on http://127.0.0.1:${port}\n`);
});
