#!/usr/bin/env node
// Limpeza opt-in SOMENTE de artefatos gerados em caminhos conhecidos.
import path from "node:path";
import { rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const paths = [
  "apps/desktop/dist",
  "apps/desktop/.vite",
  "apps/desktop/src-tauri/target",
];
if (process.argv.includes("--deep")) paths.push("target");
for (const relative of paths) {
  const target = path.resolve(root, relative);
  if (!target.startsWith(root + path.sep)) throw new Error("Caminho fora do projeto");
  await rm(target, { recursive: true, force: true });
  process.stdout.write(`Artefato gerado removido: ${relative}\n`);
}
process.stdout.write("Arquivos-fonte, locks, .env e credenciais foram preservados.\n");
