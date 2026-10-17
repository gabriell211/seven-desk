import { writeFile, mkdir } from "node:fs/promises";
import { deflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";

const output = fileURLToPath(new URL("../apps/desktop/src-tauri/icons/", import.meta.url));

function crc32(data) {
  let crc = 0xffffffff;
  for (const value of data) {
    crc ^= value;
    for (let bit = 0; bit < 8; bit++) {
      crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(name, contents) {
  const payload = Buffer.from(contents);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(payload.length);
  const kind = Buffer.from(name);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE(crc32(Buffer.concat([kind, payload])));
  return Buffer.concat([length, kind, payload, checksum]);
}

function insideRoundedSquare(x, y) {
  if (x < 0.08 || x > 0.92 || y < 0.08 || y > 0.92) return false;
  const r = 0.19;
  const dx = Math.max(0.08 + r - x, 0, x - (0.92 - r));
  const dy = Math.max(0.08 + r - y, 0, y - (0.92 - r));
  return dx * dx + dy * dy <= r * r;
}

function insideSeven(x, y) {
  // A bold, slanted 7 with a clean top bar and diagonal stem.
  const topBar = x >= 0.27 && x <= 0.75 && y >= 0.285 && y <= 0.375;
  const slantPosition = 0.71 - (y - 0.375) * 0.54;
  const stem = y >= 0.375 && y <= 0.77 && Math.abs(x - slantPosition) < 0.051;
  return topBar || stem;
}

function createPng(size) {
  const rows = Buffer.alloc(size * (1 + size * 4));
  for (let y = 0; y < size; y++) {
    const row = y * (size * 4 + 1);
    rows[row] = 0;
    for (let x = 0; x < size; x++) {
      const p = row + 1 + x * 4;
      const px = (x + 0.5) / size;
      const py = (y + 0.5) / size;
      let squareCoverage = 0;
      let glyphCoverage = 0;
      for (let sampleY = 0; sampleY < 2; sampleY++) {
        for (let sampleX = 0; sampleX < 2; sampleX++) {
          const sx = (x + (sampleX + 0.5) / 2) / size;
          const sy = (y + (sampleY + 0.5) / 2) / size;
          if (insideRoundedSquare(sx, sy)) squareCoverage++;
          if (insideSeven(sx, sy)) glyphCoverage++;
        }
      }
      const alpha = Math.round(squareCoverage * 255 / 4);
      const f = Math.min(1, glyphCoverage / 4);
      const glow = (px + py) / 2;
      const background = [Math.round(76 + glow * 40), Math.round(105 - glow * 7), 245];
      rows[p] = Math.round(background[0] * (1 - f) + 255 * f);
      rows[p + 1] = Math.round(background[1] * (1 - f) + 255 * f);
      rows[p + 2] = Math.round(background[2] * (1 - f) + 255 * f);
      rows[p + 3] = alpha;
    }
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  return Buffer.concat([
    Buffer.from([137,80,78,71,13,10,26,10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(rows, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function createIco(png) {
  const header = Buffer.alloc(22);
  header.writeUInt16LE(1, 2); // icon
  header.writeUInt16LE(1, 4); // one image
  header[6] = 0; // 256x256 (0 is ICO's representation of 256)
  header[7] = 0;
  header.writeUInt16LE(1, 10);
  header.writeUInt16LE(32, 12);
  header.writeUInt32LE(png.length, 14);
  header.writeUInt32LE(22, 18);
  return Buffer.concat([header, png]);
}

function createIcns(png256, png512) {
  const record = (type, png) => {
    const h = Buffer.alloc(8);
    h.write(type, 0, "ascii");
    h.writeUInt32BE(8 + png.length, 4);
    return Buffer.concat([h, png]);
  };
  const records = Buffer.concat([record("ic08", png256), record("ic09", png512)]);
  const header = Buffer.alloc(8);
  header.write("icns");
  header.writeUInt32BE(8 + records.length, 4);
  return Buffer.concat([header, records]);
}

await mkdir(output, { recursive: true });
const p256 = createPng(256);
const p512 = createPng(512);
await Promise.all([
  writeFile(output + "icon.png", p512),
  writeFile(output + "icon.ico", createIco(p256)),
  writeFile(output + "icon.icns", createIcns(p256, p512)),
]);
process.stdout.write("Seven Desk native icons generated.\n");
