#!/usr/bin/env node
// Real native-server HTTP integration with synthetic, ephemeral Ed25519 identities.
// This does not certify native desktop capture, remote input, WAN, or installers.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash, generateKeyPairSync, randomUUID, sign } from "node:crypto";
import { once } from "node:events";
import { mkdir, writeFile } from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const binary = path.join(root, "target", "debug", process.platform === "win32" ? "seven-server.exe" : "seven-server");
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const checks = [];

function identity() {
  const { privateKey, publicKey } = generateKeyPairSync("ed25519");
  const spki = publicKey.export({ format: "der", type: "spki" });
  assert.equal(spki.length, 44, "Unexpected Ed25519 SPKI");
  const raw = spki.subarray(12);
  const hex = createHash("sha256").update(raw).digest("hex").slice(0, 32);
  const deviceId = [hex.slice(0, 8), hex.slice(8, 12), hex.slice(12, 16), hex.slice(16, 20), hex.slice(20)].join("-");
  return { privateKey, deviceId, publicKeyHex: raw.toString("hex") };
}

function signed(peer, method, route, payload) {
  const body = payload === undefined ? Buffer.alloc(0) : Buffer.isBuffer(payload) ? payload : Buffer.from(JSON.stringify(payload));
  const nonce = randomUUID(), timestamp = Date.now();
  const digest = createHash("sha256").update(body).digest("hex");
  const message = ["seven-desk/request/v1", method, route, timestamp, nonce, digest].join("\n");
  return {
    method,
    headers: {
      "x-seven-key": peer.publicKeyHex,
      "x-seven-signature": sign(null, Buffer.from(message), peer.privateKey).toString("hex"),
      "x-seven-nonce": nonce,
      "x-seven-timestamp": String(timestamp),
      "content-type": Buffer.isBuffer(payload) ? "image/png" : "application/json",
    },
    ...(payload === undefined ? {} : { body }),
  };
}

async function port() {
  const listener = net.createServer();
  listener.listen(0, "127.0.0.1");
  await once(listener, "listening");
  const address = listener.address();
  assert.ok(address && typeof address !== "string");
  await new Promise((resolve, reject) => listener.close((error) => error ? reject(error) : resolve()));
  return address.port;
}

async function stop(child) {
  if (!child || child.exitCode !== null) return;
  const closed = once(child, "close").catch(() => {});
  child.kill();
  await Promise.race([closed, sleep(3000)]);
  if (child.exitCode === null) child.kill("SIGKILL");
}

async function verify(name, test) {
  const start = Date.now();
  try {
    await test();
    checks.push({ name, result: "pass", elapsed_ms: Date.now() - start });
  } catch (error) {
    checks.push({ name, result: "fail", elapsed_ms: Date.now() - start, detail: String(error?.message ?? error) });
    throw error;
  }
}

async function main() {
  if (!process.argv.includes("--skip-build")) {
    const result = spawnSync("cargo", ["build", "--locked", "-p", "seven-server"], { cwd: root, stdio: "inherit", timeout: 300_000 });
    if (result.error) throw result.error;
    assert.equal(result.status, 0, "Unable to compile Rust server");
  }
  const base = "http://127.0.0.1:" + await port();
  const actor = identity(), host = identity(), outsider = identity();
  const sessionId = randomUUID();
  const url = (suffix) => "/v1/sessions/" + sessionId + suffix;
  const invite = { protocol_version: 1, session_id: sessionId, operator: actor.deviceId, host: host.deviceId,
    requested: ["screen", "input"], expires_at_ms: Date.now() + 120_000 };
  // Tiny valid PNG fixture; no real desktop content is captured.
  const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/lu0AAAAASUVORK5CYII=", "base64");
  let spawnError;
  const child = spawn(binary, [], {
    cwd: root, stdio: "ignore",
    env: { ...process.env, SEVEN_BIND: base.replace("http://", ""), RUST_LOG: "seven_server=error" },
  });
  child.on("error", (error) => { spawnError = error; });
  async function call(peer, method, route, payload, status) {
    const response = await fetch(base + route, { ...signed(peer, method, route, payload), signal: AbortSignal.timeout(5000) });
    assert.equal(response.status, status, method + " " + route + ": HTTP " + response.status + " expected " + status);
    if ([202, 204].includes(status)) return null;
    if (response.headers.get("content-type")?.includes("application/json")) return response.json();
    return response.arrayBuffer();
  }
  try {
    let ready = false;
    for (let attempt = 0; attempt < 100; attempt++) {
      if (spawnError) throw spawnError;
      if (child.exitCode !== null) throw new Error("Rust server exited early");
      try {
        const response = await fetch(base + "/healthz", { signal: AbortSignal.timeout(300) });
        if (response.ok) {
          const health = await response.json();
          assert.equal(health.production_ready, false);
          assert.equal(health.protocol, 1);
          ready = true;
          break;
        }
      } catch (error) {
        if (error instanceof assert.AssertionError) throw error;
      }
      await sleep(100);
    }
    assert.ok(ready, "Real Rust server failed to start");
    await verify("registration-and-presence", async () => {
      for (const peer of [actor, outsider]) {
        const registration = await call(peer, "POST", "/v1/devices", {}, 200);
        assert.equal(registration.device_id, peer.deviceId);
      }
      await call(actor, "POST", "/v1/sessions", invite, 401); // Offline host
      const registration = await call(host, "POST", "/v1/devices", {}, 200);
      assert.equal(registration.device_id, host.deviceId);
    });
    await verify("signed-payload-tampering-and-replay", async () => {
      const route = "/v1/devices/heartbeat", proof = signed(host, "POST", route, {});
      // Session creation authenticates before parsing JSON; this proves that
      // a modified body invalidates the Ed25519 signature (not just JSON shape).
      const protectedRoute = "/v1/sessions";
      const protectedProof = signed(actor, "POST", protectedRoute, invite);
      const tampered = await fetch(base + protectedRoute, { ...protectedProof, body: Buffer.from('{"tampered":true}') });
      assert.equal(tampered.status, 401);
      assert.equal((await fetch(base + route, proof)).status, 200);
      assert.equal((await fetch(base + route, proof)).status, 401);
    });
    await verify("scoped-inbox-and-consent", async () => {
      const receipt = await call(actor, "POST", "/v1/sessions", invite, 201);
      assert.equal(receipt.status, "awaiting_consent");
      const inbox = await call(host, "GET", "/v1/sessions/inbox", undefined, 200);
      assert.ok(inbox.some((entry) => entry.session_id === sessionId));
      const outside = await call(outsider, "GET", "/v1/sessions/inbox", undefined, 200);
      assert.ok(outside.every((entry) => entry.session_id !== sessionId));
      await call(actor, "GET", url("/frame"), undefined, 403);
      await call(actor, "POST", url("/input"), { type: "move", x: 50, y: 50 }, 403);
      await call(actor, "POST", url("/decision"), { approve: true, granted: ["screen"] }, 403);
      await call(outsider, "POST", url("/decision"), { approve: true, granted: ["screen"] }, 403);
    });
    await verify("authorization-and-permission-isolation", async () => {
      const granted = await call(host, "POST", url("/decision"), { approve: true, granted: ["screen", "input"] }, 200);
      assert.equal(granted.status, "active");
      await call(actor, "POST", url("/keyboard"), { type: "character", value: "a" }, 403);
      await call(outsider, "GET", url("/frame"), undefined, 403);
      await call(actor, "GET", url("/frame"), undefined, 404);
    });
    await verify("frame-and-input-relay-with-bounded-drain", async () => {
      await call(host, "POST", url("/frame"), png, 204);
      const pixels = await call(actor, "GET", url("/frame"), undefined, 200);
      assert.deepEqual(Buffer.from(pixels), png);
      await call(actor, "POST", url("/input"), { type: "move", x: 5000, y: 5000 }, 202);
      const events = await call(host, "GET", url("/input"), undefined, 200);
      assert.equal(events.length, 1);
      assert.equal(events[0].x, 5000);
      assert.deepEqual(await call(host, "GET", url("/input"), undefined, 200), []);
    });
    await verify("host-pause-clears-frame-and-input", async () => {
      const paused = await call(host, "POST", url("/screen-delivery"), { enabled: false }, 200);
      assert.equal(paused.enabled, false);
      await call(actor, "GET", url("/frame"), undefined, 403);
      await call(actor, "POST", url("/input"), { type: "move", x: 1, y: 1 }, 403);
      await call(host, "POST", url("/frame"), png, 403);
      await call(host, "POST", url("/screen-delivery"), { enabled: true }, 200);
      await call(actor, "GET", url("/frame"), undefined, 404);
    });
    await verify("termination-and-revocation", async () => {
      await call(outsider, "DELETE", url(""), undefined, 403);
      const ended = await call(host, "DELETE", url(""), undefined, 200);
      assert.equal(ended.status, "ended");
      await call(actor, "GET", url("/frame"), undefined, 403);
      await call(actor, "POST", url("/input"), { type: "click", x: 0, y: 0, button: "left" }, 403);
      const active = await call(actor, "GET", "/v1/sessions/operator/active", undefined, 200);
      assert.ok(active.every((entry) => entry.session_id !== sessionId));
    });
  } finally {
    await stop(child);
  }
}

let failure;
try { await main(); }
catch (error) {
  failure = error;
  process.stderr.write("Session integration failed: " + String(error?.stack ?? error) + "\n");
} finally {
  const report = {
    schema_version: 1, suite: "seven-desk-signed-session-api", commit: process.env.GITHUB_SHA || null,
    created_at: new Date().toISOString(), platform: process.platform,
    scope: "real Rust HTTP signaling, synthetic client identities, loopback only",
    native_desktop_e2e_verified: false, result: failure ? "fail" : "pass", checks,
  };
  try {
    const directory = path.join(root, "artifacts", "qa");
    await mkdir(directory, { recursive: true });
    await writeFile(path.join(directory, "session-api-" + process.platform + ".json"), JSON.stringify(report, null, 2) + "\n");
  } catch (error) { failure ||= error; }
  process.stdout.write("Seven Desk signed session API: " + report.result + " (" + checks.length + " checks)\n");
}
if (failure) process.exitCode = 1;
