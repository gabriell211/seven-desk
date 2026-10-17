#!/usr/bin/env node
// Smoke test real de inicializacao do Seven Desk sem Docker e sem credenciais.
// Node 22+; inicia o binario compilado, testa HTTP real e encerra o processo.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const binary = path.join(root, "target", "debug", process.platform === "win32" ? "seven-server.exe" : "seven-server");
const skipBuild = process.argv.includes("--skip-build");
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function freeLoopbackPort() {
  const server = net.createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  await new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
  return address.port;
}

async function awaitHealth(url, child) {
  for (let attempt = 0; attempt < 100; attempt++) {
    if (child.exitCode !== null) throw new Error(`Servidor encerrou prematuramente: ${child.exitCode}`);
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(300) });
      if (response.ok) return response.json();
    } catch {
      // The native server may still be binding the loopback socket.
    }
    await wait(100);
  }
  throw new Error("Servidor nao respondeu em localhost dentro do prazo");
}

async function stop(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await Promise.race([once(child, "close").catch(() => {}), wait(3000)]);
  if (child.exitCode === null) child.kill("SIGKILL");
}

function validateFixtures(data) {
  assert.equal(data.schema_version, 1);
  assert.equal(data.devices.length, 2);
  const devices = new Set(data.devices.map((device) => {
    assert.match(device.device_id, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
    return device.device_id;
  }));
  assert.equal(devices.size, 2, "Ids ficticios nao podem colidir");
  for (const testCase of data.session_cases) {
    assert.ok(testCase.requested.includes("screen"));
    assert.ok(testCase.granted.every((capability) => testCase.requested.includes(capability)),
      "Fixture nao pode escalar permissoes");
  }
  assert.ok(!/private_key|password|secret|access_token/i.test(JSON.stringify(data)));
}

async function main() {
  const fixture = JSON.parse(await readFile(path.join(root, "tests/fixtures/devices-and-sessions.json"), "utf8"));
  validateFixtures(fixture);
  if (!skipBuild) {
    const build = spawnSync("cargo", ["build", "--locked", "-p", "seven-server"], {
      cwd: root, stdio: "inherit", timeout: 300_000,
    });
    if (build.error) throw build.error;
    assert.equal(build.status, 0, "Falha ao compilar o backend Rust");
  }

  const port = await freeLoopbackPort();
  const env = {
    ...process.env,
    SEVEN_BIND: `127.0.0.1:${port}`,
    // O backend atual e in-memory e nao deve depender de um DB online.
    DATABASE_URL: "postgres://127.0.0.1:1/seven-desk-fictitious-unavailable",
    RUST_LOG: "seven_server=warn",
  };
  let service;
  try {
    service = spawn(binary, [], { cwd: root, env, stdio: ["ignore", "pipe", "pipe"] });
    service.on("error", (error) => { process.stderr.write(String(error) + "\n"); });
    const base = `http://127.0.0.1:${port}`;
    const health = await awaitHealth(`${base}/healthz`, service);
    assert.equal(health.status, "ok");
    assert.equal(health.production_ready, false);
    assert.ok(health.protocol > 0);
    const unsigned = await fetch(`${base}/v1/devices`, {
      method: "POST", headers: { "content-type": "application/json" }, body: "{}",
    });
    assert.equal(unsigned.status, 401, "API nao deve aceitar cadastro sem assinatura");
    const invalid = await fetch(`${base}/v1/devices`, {
      method: "POST", headers: { "content-type": "application/json" }, body: "bad-payload",
    });
    assert.equal(invalid.status, 400, "API deve rejeitar payload de cadastro invalido");
    process.stdout.write("OK: fixtures, health HTTP, ausencia de banco, assinatura e payload invalido\n");
  } finally {
    await stop(service);
  }

  // Nao ha modo remoto implicito: tentar bind fora do localhost deve falhar.
  const unsafe = spawn(binary, [], {
    cwd: root, env: { ...process.env, SEVEN_BIND: "0.0.0.0:0" }, stdio: "ignore",
  });
  const result = await Promise.race([
    once(unsafe, "close").then(([exitCode]) => exitCode),
    wait(4000).then(() => { unsafe.kill(); throw new Error("Bind externo nao foi recusado"); }),
  ]);
  assert.notEqual(result, 0, "Servidor nao pode ouvir em 0.0.0.0");
  process.stdout.write("OK: bind externo bloqueado; smoke sem Docker concluido\n");
}

main().catch((error) => {
  process.stderr.write(`FALHA no smoke: ${error.stack || String(error)}\n`);
  process.exitCode = 1;
});
