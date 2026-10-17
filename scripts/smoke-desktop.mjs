#!/usr/bin/env node
// Smoke de HTML/asset real do Vite preview em localhost. Nao simula sessao.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { access } from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const desktop = path.join(root, "apps/desktop");
const vite = path.join(root, "node_modules/vite/bin/vite.js");
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function freePort() {
  const listener = net.createServer();
  listener.listen(0, "127.0.0.1");
  await once(listener, "listening");
  const addr = listener.address();
  assert.ok(addr && typeof addr !== "string");
  await new Promise((resolve, reject) => listener.close((error) => error ? reject(error) : resolve()));
  return addr.port;
}

async function main() {
  await access(path.join(desktop, "dist/index.html"));
  await access(vite);
  const port = await freePort();
  const child = spawn(process.execPath, [vite, "preview", "--host", "127.0.0.1", "--port", String(port), "--strictPort"], {
    cwd: desktop,
    stdio: "ignore",
  });
  try {
    let html;
    for (let i = 0; i < 100; i++) {
      if (child.exitCode !== null) throw new Error(`Preview encerrou: ${child.exitCode}`);
      try {
        const response = await fetch(`http://127.0.0.1:${port}/`, {
          signal: AbortSignal.timeout(300),
        });
        if (response.ok) { html = await response.text(); break; }
      } catch {
        // Processo ainda iniciando.
      }
      await wait(100);
    }
    assert.ok(html, "Vite preview nao respondeu em localhost");
    assert.match(html, /<title>Seven Desk<\/title>/);
    assert.match(html, /assets\//, "Preview nao esta entregando HTML de build");
    const missing = await fetch(`http://127.0.0.1:${port}/brand-mark.svg`);
    assert.equal(missing.status, 200);
    assert.match(await missing.text(), /<svg/i);
    process.stdout.write("OK: bundle React, Vite preview e asset de marca respondem em localhost\n");
  } finally {
    if (child.exitCode === null) child.kill();
    await Promise.race([once(child, "close").catch(() => {}), wait(3000)]);
  }
}

main().catch((error) => {
  process.stderr.write(`FALHA no frontend smoke: ${error.stack || String(error)}\n`);
  process.exitCode = 1;
});
