import { mkdir, writeFile } from "node:fs/promises";
import { deflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";

// The native counterpart of public/brand-mark.svg. Keep the tile bounds,
// seven polygon, and gradient stops in sync with the vector artwork.
const output = fileURLToPath(new URL("../apps/desktop/src-tauri/icons/", import.meta.url));
const seven = [
  [0.28, 0.295], [0.755, 0.295], [0.755, 0.405],
  [0.49, 0.735], [0.36, 0.735], [0.63, 0.405], [0.28, 0.405],
];

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

function chunk(name, payload) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(payload.length);
  const kind = Buffer.from(name);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE(crc32(Buffer.concat([kind, payload])));
  return Buffer.concat([length, kind, payload, checksum]);
}

function inTile(x, y) {
  const left = 0.055;
  const top = 0.055;
  const right = 0.945;
  const bottom = 0.945;
  const radius = 0.23;
  const nearestX = Math.max(left + radius, Math.min(x, right - radius));
  const nearestY = Math.max(top + radius, Math.min(y, bottom - radius));
  const dx = x - nearestX;
  const dy = y - nearestY;
  return dx * dx + dy * dy <= radius * radius;
}

function inSeven(x, y) {
  let inside = false;
  for (let i = 0, j = seven.length - 1; i < seven.length; j = i++) {
    const [xi, yi] = seven[i];
    const [xj, yj] = seven[j];
    if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) {
      inside = !inside;
    }
  }
  return inside;
}

function createPng(size) {
  const stride = 1 + size * 4;
  const rows = Buffer.alloc(size * stride);
  const samples = size <= 32 ? 6 : 4;
  const sampleCount = samples * samples;
  for (let y = 0; y < size; y++) {
    const row = y * stride;
    for (let x = 0; x < size; x++) {
      let tileCoverage = 0;
      let sevenCoverage = 0;
      for (let sy = 0; sy < samples; sy++) {
        for (let sx = 0; sx < samples; sx++) {
          const px = (x + (sx + 0.5) / samples) / size;
          const py = (y + (sy + 0.5) / samples) / size;
          if (inTile(px, py)) {
            tileCoverage++;
            if (inSeven(px, py)) sevenCoverage++;
          }
        }
      }
      const offset = row + 1 + x * 4;
      if (tileCoverage === 0) continue;
      const gradient = Math.max(0, Math.min(1, ((x + y + 1) / size) / 2));
      const white = sevenCoverage / tileCoverage;
      const red = 76 + (119 - 76) * gradient;
      const green = 116 + (66 - 116) * gradient;
      const blue = 255 + (239 - 255) * gradient;
      rows[offset] = Math.round(red * (1 - white) + 255 * white);
      rows[offset + 1] = Math.round(green * (1 - white) + 255 * white);
      rows[offset + 2] = Math.round(blue * (1 - white) + 255 * white);
      rows[offset + 3] = Math.round((tileCoverage / sampleCount) * 255);
    }
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; // RGBA, eight bits per channel.
  ihdr[9] = 6;
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(rows, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function createIco(images) {
  const header = Buffer.alloc(6 + images.length * 16);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(images.length, 4);
  let offset = header.length;
  for (const [index, [size, png]] of images.entries()) {
    const entry = 6 + index * 16;
    header[entry] = size === 256 ? 0 : size;
    header[entry + 1] = size === 256 ? 0 : size;
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(png.length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += png.length;
  }
  return Buffer.concat([header, ...images.map(([, png]) => png)]);
}

function createIcns(images) {
  const types = new Map([[16, "icp4"], [32, "icp5"], [64, "icp6"], [128, "ic07"], [256, "ic08"], [512, "ic09"]]);
  const records = images.map(([size, png]) => {
    const header = Buffer.alloc(8);
    header.write(types.get(size), 0, "ascii");
    header.writeUInt32BE(png.length + 8, 4);
    return Buffer.concat([header, png]);
  });
  const header = Buffer.alloc(8);
  header.write("icns");
  header.writeUInt32BE(8 + records.reduce((total, record) => total + record.length, 0), 4);
  return Buffer.concat([header, ...records]);
}

function createBmp(width, height, pixel) {
  const stride = (width * 3 + 3) & ~3;
  const bmp = Buffer.alloc(54 + stride * height);
  bmp.write("BM", 0, "ascii");
  bmp.writeUInt32LE(bmp.length, 2);
  bmp.writeUInt32LE(54, 10);
  bmp.writeUInt32LE(40, 14);
  bmp.writeInt32LE(width, 18);
  bmp.writeInt32LE(height, 22);
  bmp.writeUInt16LE(1, 26);
  bmp.writeUInt16LE(24, 28);
  bmp.writeUInt32LE(stride * height, 34);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const [r, g, b] = pixel(x, y);
      const index = 54 + (height - 1 - y) * stride + x * 3;
      bmp[index] = Math.round(b);
      bmp[index + 1] = Math.round(g);
      bmp[index + 2] = Math.round(r);
    }
  }
  return bmp;
}

function installerPixel(x, y, width, height, logo) {
  const t = (x / width) * 0.5 + (y / height) * 0.5;
  const base = [18 + t * 15, 24 + t * 16, 40 + t * 30];
  const [left, top, size] = logo;
  const lx = (x - left) / size;
  const ly = (y - top) / size;
  if (lx >= 0 && lx <= 1 && ly >= 0 && ly <= 1 && inTile(lx, ly)) {
    const blend = (lx + ly) / 2;
    if (inSeven(lx, ly)) return [246, 248, 255];
    return [76 + blend * 43, 116 - blend * 50, 255 - blend * 16];
  }
  // Fine diagonal trails and a bright seven-segment line anchor the artwork.
  const trail = (x + y * 0.68) % 58;
  if (trail < 1.2 && y > height * 0.52 && x > width * 0.25) {
    return [base[0] + 20, base[1] + 24, base[2] + 48];
  }
  if (x > width * 0.13 && x < width * 0.87 && y > height * 0.83 && y < height * 0.845) {
    return [89 + t * 25, 122 - t * 20, 255 - t * 12];
  }
  return base;
}

await mkdir(output, { recursive: true });
const sizes = [16, 24, 32, 48, 64, 128, 256, 512];
const pngs = new Map(sizes.map((size) => [size, createPng(size)]));
await Promise.all([
  writeFile(output + "icon.png", pngs.get(512)),
  writeFile(output + "icon.ico", createIco([16, 24, 32, 48, 64, 128, 256].map((size) => [size, pngs.get(size)]))),
  writeFile(output + "icon.icns", createIcns([16, 32, 64, 128, 256, 512].map((size) => [size, pngs.get(size)]))),
  writeFile(output + "installer-header.bmp", createBmp(150, 57, (x, y) =>
    installerPixel(x, y, 150, 57, [101, 5, 46]))),
  writeFile(output + "installer-sidebar.bmp", createBmp(164, 314, (x, y) =>
    installerPixel(x, y, 164, 314, [13, 54, 138]))),
]);
process.stdout.write("Seven Desk native icons and installer artwork generated.\n");
