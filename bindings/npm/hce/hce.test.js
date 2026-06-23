import { describe, it, expect, beforeAll } from "vitest";
import { readFileSync } from "fs";
import { randomBytes } from "crypto";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

let HceCodec;
let initSync;

beforeAll(async () => {
  const wasmBytes = readFileSync(join(__dirname, "index_bg.wasm"));
  const mod = await import(join(__dirname, "index.js"));
  initSync = mod.initSync || mod.default;
  HceCodec = mod.HceCodec;
  initSync(wasmBytes);
});

const KEY = "hce-kat-standard-key-32-bytes!!";
const KEY_HEX = Buffer.from(KEY).toString("hex");
const UUID_BYTES = new Uint8Array([
  1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39,
]);

describe("HceCodec constructor", () => {
  it("creates with hex key", () => {
    const h = new HceCodec(KEY_HEX, "universal", "sealed");
    expect(h).toBeDefined();
  });

  it("creates with raw key", () => {
    const h = new HceCodec(KEY, "universal", "sealed");
    expect(h).toBeDefined();
  });

  it("creates plain mode without key", () => {
    const h = new HceCodec(null, "universal", "plain");
    expect(h).toBeDefined();
  });

  it("rejects invalid level", () => {
    expect(() => new HceCodec(KEY, "invalid", "sealed")).toThrow();
  });

  it("rejects invalid mode", () => {
    expect(() => new HceCodec(KEY, "universal", "invalid")).toThrow();
  });
});

describe("Sealed mode", () => {
  let h;
  beforeAll(() => {
    h = new HceCodec(KEY_HEX, "universal", "sealed");
  });

  it("encodes and decodes roundtrip", () => {
    const enc = h.encode(UUID_BYTES);
    const dec = h.decode(enc);
    expect(new Uint8Array(dec)).toEqual(UUID_BYTES);
  });

  it("output contains no prefix", () => {
    const enc = h.encode(UUID_BYTES);
    expect(enc).not.toMatch(/^K/);
  });

  it("empty input encodes", () => {
    const enc = h.encode(new Uint8Array(0));
    expect(enc.length).toBeGreaterThan(0);
    expect(h.decode(enc)).toBeDefined();
  });
});

describe("Bit width variants", () => {
  for (const bw of [32, 64, 96, 128]) {
    it(`bit width ${bw} roundtrip`, () => {
      const h = new HceCodec(KEY_HEX, "universal", "sealed").withBitWidth(bw);
      const byteLen = Math.ceil(bw / 8);
      const data = new Uint8Array(Array.from({ length: byteLen }, (_, i) => (i * 17 + 1) % 256));
      const enc = h.encode(data);
      const dec = new Uint8Array(h.decode(enc));
      expect(dec.slice(16 - byteLen)).toEqual(data);
    });
  }
});

describe("Chunk modes", () => {
  it("none chunk has no hyphens", () => {
    const h = new HceCodec(KEY_HEX, "universal", "sealed").withChunkNone();
    const enc = h.encode(UUID_BYTES);
    expect(enc).not.toContain("-");
  });
});

describe("Custom cipher", () => {
  it("shuffle produces different output", () => {
    const h1 = new HceCodec(KEY_HEX, "universal", "sealed");
    const h2 = new HceCodec(KEY_HEX, "universal", "sealed").withCipher("shuffle");
    const e1 = h1.encode(UUID_BYTES);
    const e2 = h2.encode(UUID_BYTES);
    expect(e1).not.toBe(e2);
    expect(new Uint8Array(h2.decode(e2))).toEqual(UUID_BYTES);
  });
});

describe("Recovery", () => {
  it("recovers valid input", () => {
    const h = new HceCodec(KEY_HEX, "universal", "sealed");
    const enc = h.encode(UUID_BYTES);
    const result = h.recover(enc);
    expect(result).toBeInstanceOf(Uint8Array);
  });
});

describe("KAT vectors", () => {
  const levels = ["universal", "eu", "en", "numeric"];

  for (const level of levels) {
    it(`${level} KAT`, () => {
      const kat = JSON.parse(
        readFileSync(join(__dirname, "..", "..", "..", "shared", "kat", `${level}.json`), "utf8")
      );
      for (const v of kat.vectors) {
        const uuidBytes = new Uint8Array(
          v.uuid.match(/.{2}/g).map((h) => parseInt(h, 16))
        );
        const mode = v.mode;
        const key = mode === "plain" ? null : KEY_HEX;
        const h = new HceCodec(key, level, mode);

        const enc = h.encode(uuidBytes);
        expect(enc, `${level}[${v.index}] ${mode}`).toBe(v.hce);

        const dec = new Uint8Array(h.decode(v.hce));
        expect(dec, `${level}[${v.index}] ${mode} roundtrip`).toEqual(uuidBytes);
      }
    });
  }
});
