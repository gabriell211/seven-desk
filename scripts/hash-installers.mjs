import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { readFile, readdir, stat, writeFile } from "node:fs/promises";
import { extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Usage: node scripts/hash-installers.mjs [--verify] [bundle-directory]
// Run this on locally built packages or on a release staging directory.
const defaultBundle = fileURLToPath(new URL("../apps/desktop/src-tauri/target/release/bundle/", import.meta.url));
const args = process.argv.slice(2);
const verify = args.includes("--verify");
const paths = args.filter((arg) => arg !== "--verify");
if (paths.length > 1 || args.some((arg) => arg.startsWith("--") && arg !== "--verify")) {
  throw new Error("Usage: node scripts/hash-installers.mjs [--verify] [bundle-directory]");
}
const bundle = resolve(paths[0] ?? defaultBundle);
if (!(await stat(bundle).catch(() => null))?.isDirectory()) {
  throw new Error(`Bundle directory does not exist: ${bundle}`);
}

async function collect(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await collect(path));
    else if (entry.isFile() && [".exe", ".msi", ".dmg", ".deb", ".rpm", ".appimage"].includes(extname(entry.name).toLowerCase())) {
      files.push(path);
    }
  }
  return files;
}

const files = (await collect(bundle)).sort((a, b) => {
  const left = relative(bundle, a).replaceAll("\\", "/");
  const right = relative(bundle, b).replaceAll("\\", "/");
  return left < right ? -1 : left > right ? 1 : 0;
});
if (files.length === 0) throw new Error(`No installer packages found in ${bundle}`);

const lines = [];
for (const path of files) {
  const name = relative(bundle, path).replaceAll("\\", "/");
  if (/[\r\n]/.test(name)) throw new Error(`Unsupported filename: ${name}`);
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  lines.push(`${hash.digest("hex")}  ${name}`);
}
const expected = `${lines.join("\n")}\n`;
const manifest = join(bundle, "SHA256SUMS.txt");
if (verify) {
  if (await readFile(manifest, "utf8") !== expected) throw new Error(`Installer hashes do not match ${manifest}`);
  process.stdout.write(`Verified ${files.length} installer package(s) against ${manifest}\n`);
} else {
  await writeFile(manifest, expected, "utf8");
  process.stdout.write(`Wrote ${manifest} for ${files.length} installer package(s)\n`);
}
