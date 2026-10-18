#!/usr/bin/env node
// Manual, consented loopback QA with an ephemeral operator. No key or pixels
// are written to disk; mouse movement requires an explicit stdin command.
import { createHash, generateKeyPairSync, randomUUID, sign } from "node:crypto";
import { createInterface } from "node:readline";

const target = process.argv[2];
if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(target ?? "")) {
  throw new Error("Uso: node scripts/live-local-test.mjs <ID do host>");
}
const base = "http://127.0.0.1:43821";
const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const rawKey = publicKey.export({ format: "der", type: "spki" }).subarray(12);
const digest = createHash("sha256").update(rawKey).digest("hex").slice(0, 32);
const operator = [digest.slice(0, 8), digest.slice(8, 12), digest.slice(12, 16), digest.slice(16, 20), digest.slice(20)].join("-");
let sessionId;
let closed = false;
let lastState;
let finish;
const finished = new Promise((resolve) => { finish = resolve; });
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function request(method, route, payload) {
  const body = payload === undefined ? Buffer.alloc(0) : Buffer.from(JSON.stringify(payload));
  const nonce = randomUUID();
  const timestamp = Date.now();
  const bodyHash = createHash("sha256").update(body).digest("hex");
  const message = ["seven-desk/request/v1", method, route, timestamp, nonce, bodyHash].join("\n");
  const response = await fetch(base + route, {
    method,
    headers: {
      "x-seven-key": rawKey.toString("hex"),
      "x-seven-signature": sign(null, Buffer.from(message), privateKey).toString("hex"),
      "x-seven-nonce": nonce,
      "x-seven-timestamp": String(timestamp),
      "content-type": "application/json",
    },
    ...(payload === undefined ? {} : { body }),
    signal: AbortSignal.timeout(5000),
  });
  if (!response.ok) throw new Error(`${method} ${route}: HTTP ${response.status}`);
  return response;
}

async function end() {
  if (closed) return;
  closed = true;
  if (sessionId) {
    try { await request("DELETE", `/v1/sessions/${sessionId}`); }
    catch { /* An expired or host-terminated invitation needs no cleanup. */ }
  }
  finish();
  process.exitCode = 0;
}

async function main() {
  await request("POST", "/v1/devices", {});
  console.log(`Operador temporário: ${operator}`);
  console.log(`Aguardando ${target} registrar-se no servidor local...`);
  const deadline = Date.now() + 90_000;
  while (Date.now() < deadline) {
    const response = await request("GET", `/v1/devices/${target}/presence`);
    if ((await response.json()).online) break;
    await pause(2000);
  }
  const response = await request("GET", `/v1/devices/${target}/presence`);
  if (!(await response.json()).online) throw new Error("Host não se registrou no servidor local em 90 segundos");
  sessionId = randomUUID();
  await request("POST", "/v1/sessions", {
    protocol_version: 1,
    session_id: sessionId,
    operator,
    host: target,
    requested: ["screen", "input", "keyboard"],
    expires_at_ms: Date.now() + 120_000,
  });
  console.log(`Convite enviado: ${sessionId}`);
  console.log("No host: revise as permissões na janela de atendimento e clique em Aceitar.");
  console.log("Comandos neste terminal: status, frame, move, end. Nada move o cursor automaticamente.");

  const heartbeat = setInterval(() => { void request("POST", "/v1/devices/heartbeat", {}).catch(() => {}); }, 30_000);
  const timer = setInterval(() => {
    void request("GET", `/v1/sessions/${sessionId}`).then(async (current) => {
      const state = await current.json();
      if (state !== lastState) { console.log(`Sessão: ${state}`); lastState = state; }
    }).catch((error) => console.log(`Estado indisponível: ${error.message}`));
  }, 2000);
  const input = createInterface({ input: process.stdin, output: process.stdout, terminal: false });
  input.on("line", (line) => {
    void (async () => {
      try {
        switch (line.trim().toLowerCase()) {
          case "status": {
            const current = await request("GET", `/v1/sessions/${sessionId}`);
            console.log(`Sessão: ${await current.json()}`);
            break;
          }
          case "frame": {
            const frame = await request("GET", `/v1/sessions/${sessionId}/frame`);
            console.log(`Quadro autorizado recebido: ${(await frame.arrayBuffer()).byteLength} bytes (descartado)`);
            break;
          }
          case "move": {
            await request("POST", `/v1/sessions/${sessionId}/input`, { type: "move", x: 7000, y: 5000 });
            console.log("Movimento solicitado; o host só aplica se o mouse estiver habilitado localmente.");
            break;
          }
          case "end":
            await end();
            input.close();
            break;
          default: console.log("Use: status, frame, move ou end.");
        }
      } catch (error) { console.log(`Teste: ${error.message}`); }
    })();
  });
  let timeoutId;
  await Promise.race([finished, new Promise((resolve) => { timeoutId = setTimeout(resolve, 120_000); })]);
  clearTimeout(timeoutId);
  clearInterval(heartbeat);
  clearInterval(timer);
  await end();
  input.close();
}

main().catch((error) => { console.error(`Teste local interrompido: ${error.message}`); process.exitCode = 1; });
