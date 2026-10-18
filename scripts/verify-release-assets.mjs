import { readdir, stat } from "node:fs/promises";
import { extname, join, resolve } from "node:path";

const directory = resolve(process.argv[2] ?? "release-assets");
const expected = new Map([[".exe", 1], [".msi", 1], [".dmg", 2], [".deb", 1], [".rpm", 1]]);
const files = await readdir(directory);
for (const [extension, count] of expected) {
  const matches = files.filter((file) => extname(file).toLowerCase() === extension);
  if (matches.length !== count) throw new Error(`Expected ${count} ${extension} file(s), got ${matches.length}`);
  for (const file of matches) {
    if (!(await stat(join(directory, file))).size) throw new Error(`Empty package: ${file}`);
  }
}
if (files.length !== 6) throw new Error(`Unexpected release files: ${files.join(", ")}`);
process.stdout.write("Verified all six Windows, macOS, and Linux packages.\n");
