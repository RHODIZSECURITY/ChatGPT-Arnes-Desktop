import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SIZE = 256;
const BYTES_PER_PIXEL = 4;
const XOR_SIZE = SIZE * SIZE * BYTES_PER_PIXEL;
const AND_ROW_BYTES = Math.ceil(SIZE / 32) * 4;
const AND_SIZE = AND_ROW_BYTES * SIZE;
const BITMAP_INFO_SIZE = 40;
const IMAGE_SIZE = BITMAP_INFO_SIZE + XOR_SIZE + AND_SIZE;
const ICON_HEADER_SIZE = 6;
const ICON_ENTRY_SIZE = 16;
const IMAGE_OFFSET = ICON_HEADER_SIZE + ICON_ENTRY_SIZE;

const pixels = new Uint8Array(SIZE * SIZE * 4);

const setPixel = (x, y, [r, g, b, a]) => {
  if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) {
    return;
  }
  const offset = (y * SIZE + x) * 4;
  pixels[offset] = r;
  pixels[offset + 1] = g;
  pixels[offset + 2] = b;
  pixels[offset + 3] = a;
};

const fillRect = (x, y, width, height, color) => {
  for (let yy = y; yy < y + height; yy += 1) {
    for (let xx = x; xx < x + width; xx += 1) {
      setPixel(xx, yy, color);
    }
  }
};

const drawThickLine = (x0, y0, x1, y1, radius, color) => {
  const minX = Math.floor(Math.min(x0, x1) - radius);
  const maxX = Math.ceil(Math.max(x0, x1) + radius);
  const minY = Math.floor(Math.min(y0, y1) - radius);
  const maxY = Math.ceil(Math.max(y0, y1) + radius);
  const dx = x1 - x0;
  const dy = y1 - y0;
  const lengthSquared = dx * dx + dy * dy;

  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const t = Math.max(
        0,
        Math.min(1, ((x - x0) * dx + (y - y0) * dy) / lengthSquared),
      );
      const px = x0 + t * dx;
      const py = y0 + t * dy;
      const distanceSquared = (x - px) ** 2 + (y - py) ** 2;
      if (distanceSquared <= radius ** 2) {
        setPixel(x, y, color);
      }
    }
  }
};

const background = [9, 13, 20, 255];
const foreground = [232, 237, 247, 255];
const accent = [145, 166, 255, 255];

fillRect(0, 0, SIZE, SIZE, background);

// Geometric RHODIZ placeholder mark: a compact R plus one accent block.
fillRect(56, 50, 38, 158, foreground);
fillRect(56, 50, 102, 36, foreground);
fillRect(56, 108, 98, 34, foreground);
fillRect(144, 68, 38, 58, foreground);
drawThickLine(118, 134, 181, 202, 18, foreground);
fillRect(176, 50, 30, 30, accent);

const ico = Buffer.alloc(IMAGE_OFFSET + IMAGE_SIZE);

// ICONDIR
ico.writeUInt16LE(0, 0);
ico.writeUInt16LE(1, 2);
ico.writeUInt16LE(1, 4);

// ICONDIRENTRY. Width/height 0 means 256.
ico.writeUInt8(0, 6);
ico.writeUInt8(0, 7);
ico.writeUInt8(0, 8);
ico.writeUInt8(0, 9);
ico.writeUInt16LE(1, 10);
ico.writeUInt16LE(32, 12);
ico.writeUInt32LE(IMAGE_SIZE, 14);
ico.writeUInt32LE(IMAGE_OFFSET, 18);

// BITMAPINFOHEADER
let cursor = IMAGE_OFFSET;
ico.writeUInt32LE(BITMAP_INFO_SIZE, cursor);
ico.writeInt32LE(SIZE, cursor + 4);
ico.writeInt32LE(SIZE * 2, cursor + 8);
ico.writeUInt16LE(1, cursor + 12);
ico.writeUInt16LE(32, cursor + 14);
ico.writeUInt32LE(0, cursor + 16);
ico.writeUInt32LE(XOR_SIZE, cursor + 20);
ico.writeInt32LE(0, cursor + 24);
ico.writeInt32LE(0, cursor + 28);
ico.writeUInt32LE(0, cursor + 32);
ico.writeUInt32LE(0, cursor + 36);
cursor += BITMAP_INFO_SIZE;

// ICO BMP pixel rows are bottom-up and use BGRA byte order.
for (let y = SIZE - 1; y >= 0; y -= 1) {
  for (let x = 0; x < SIZE; x += 1) {
    const source = (y * SIZE + x) * 4;
    ico[cursor] = pixels[source + 2];
    ico[cursor + 1] = pixels[source + 1];
    ico[cursor + 2] = pixels[source];
    ico[cursor + 3] = pixels[source + 3];
    cursor += 4;
  }
}

// AND mask is all zero because this placeholder is fully opaque.
ico.fill(0, cursor, cursor + AND_SIZE);

const currentDir = dirname(fileURLToPath(import.meta.url));
const output = resolve(currentDir, "../src-tauri/icons/icon.ico");
mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, ico);

console.log(`generated ${output} (${ico.length} bytes)`);
