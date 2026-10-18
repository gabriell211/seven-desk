import { copyFile, mkdir, readdir, stat } from "node:fs/promises";
import { basename, extname, join, resolve } from "node:path";

// Copy only installable packages to one flat artifact directory.
const [sourceArg, destinationArg, extensionsArg] = process.argv.slice(2);
if (!sourceArg || !destinationArg || !extensionsArg) {
  throw new Error("Usage: node scripts/stage-release.mjs <bundle-dir> <out-dir> <extensions>");
}
const source = resolve(sourceArg);
const destination = resolve(destinationArg);
const extensions = new Set(extensionsArg.split(",").map((entry) => entry.toLowerCase()));
const files = [];
async function collect(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await collect(path);
    else if (entry.isFile() && extensions.has(extname(entry.name).toLowerCase())) files.push(path);
  }
}
await collect(source);
if (files.length !== extensions.size) {
  throw new Error(`Expected ${extensions.size} package(s) from ${source}, got ${files.length}`);
}
await mkdir(destination, { recursive: true });
for (const path of files) {
  if (!(await stat(path)).size) throw new Error(`Empty package: ${path}`);
  await copyFile(path, join(destination, basename(path)));
}
process.stdout.write(`Staged ${files.length} package(s) in ${destination}\n`);
