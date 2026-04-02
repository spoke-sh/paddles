import http from 'node:http';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import process from 'node:process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '../../..');
const webPort = Number.parseInt(process.env.PORT || '4174', 10);
const providerPort = Number.parseInt(process.env.PROVIDER_PORT || '4175', 10);

function openAiContentResponse(content) {
  return JSON.stringify({
    choices: [
      {
        message: {
          content,
        },
      },
    ],
  });
}

function openAiToolCallResponse(argumentsJson) {
  return JSON.stringify({
    choices: [
      {
        message: {
          content: null,
          tool_calls: [
            {
              id: 'call_legacy_shell',
              type: 'function',
              function: {
                name: 'select_planner_action',
                arguments: argumentsJson,
              },
            },
          ],
        },
      },
    ],
  });
}

const queuedResponses = [
  openAiContentResponse('I should inspect the local workspace before answering.'),
  openAiToolCallResponse(
    JSON.stringify({
      action: 'inspect',
      command: 'pwd',
      rationale: 'inspect the local workspace before answering',
    })
  ),
  openAiToolCallResponse(
    JSON.stringify({
      action: 'answer',
      rationale: 'the local evidence is sufficient',
    })
  ),
  openAiContentResponse(
    JSON.stringify({
      render_types: ['paragraph'],
      blocks: [
        {
          type: 'paragraph',
          text: 'Mock provider completed the turn after local inspection.',
        },
      ],
    })
  ),
];

const providerServer = http.createServer((req, res) => {
  const url = new URL(req.url || '/', `http://127.0.0.1:${providerPort}`);

  if (req.method === 'GET' && url.pathname === '/health') {
    res.writeHead(200, { 'content-type': 'application/json; charset=utf-8' });
    res.end(JSON.stringify({ status: 'ok' }));
    return;
  }

  const body =
    queuedResponses.shift() ||
    openAiContentResponse(
      JSON.stringify({
        render_types: ['paragraph'],
        blocks: [
          {
            type: 'paragraph',
            text: 'Mock provider exhausted its queued responses.',
          },
        ],
      })
    );

  res.writeHead(200, { 'content-type': 'application/json; charset=utf-8' });
  res.end(body);
});

let paddles = null;
let shuttingDown = false;

function shutdown(code = 0) {
  if (shuttingDown) {
    return;
  }
  shuttingDown = true;

  if (paddles && !paddles.killed) {
    paddles.kill('SIGTERM');
  }

  providerServer.close(() => {
    process.exit(code);
  });

  setTimeout(() => {
    if (paddles && !paddles.killed) {
      paddles.kill('SIGKILL');
    }
    process.exit(code);
  }, 5_000).unref();
}

providerServer.listen(providerPort, '127.0.0.1', () => {
  paddles = spawn(
    'nix',
    [
      'develop',
      '--command',
      'cargo',
      'run',
      '--quiet',
      '--',
      '--provider',
      'inception',
      '--provider-url',
      `http://127.0.0.1:${providerPort}`,
      '--model',
      'mercury-2',
      '--planner-provider',
      'inception',
      '--planner-model',
      'mercury-2',
      '--port',
      String(webPort),
    ],
    {
      cwd: repoRoot,
      env: {
        ...process.env,
        INCEPTION_API_KEY: process.env.INCEPTION_API_KEY || 'test-key',
      },
      stdio: ['pipe', 'inherit', 'inherit'],
    }
  );

  paddles.on('exit', (code, signal) => {
    if (!shuttingDown) {
      console.error(
        `paddles exited before the E2E harness completed (code=${code}, signal=${signal})`
      );
      shutdown(code ?? 1);
    }
  });
});

process.on('SIGINT', () => shutdown(0));
process.on('SIGTERM', () => shutdown(0));
