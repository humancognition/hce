# @humancognition/hce

Lossless, pronounceable, encrypted codec for any identifier.

## Install

```bash
npm install @humancognition/hce
```

## Usage

Node.js — zero setup, WASM inlined:

```js
import { HceCodec } from "@humancognition/hce";

const key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const hce = new HceCodec(key, "universal", "sealed");

const id = hce.encode(crypto.randomBytes(16));
const bytes = hce.decode(id);
```

Browser — one-line init:

```js
import init, { HceCodec } from "@humancognition/hce";
await init();
const hce = new HceCodec(key, "universal", "sealed");
```

## API

```ts
class HceCodec {
  constructor(key: string | null, level: string, mode: string)
  encode(data: Uint8Array): string
  decode(input: string): Uint8Array
  recover(input: string): Uint8Array
  withBitWidth(bits: number): HceCodec
  withCase(uppercase: boolean): HceCodec
  withCheckSyllables(n: number): HceCodec
  withSeparator(sep: string): HceCodec
  withChunkNone(): HceCodec
  withChunkFixed(charSize: number): HceCodec
  withChunkPattern(pattern: number[]): HceCodec
  withCipher(kind: "feistel" | "shuffle", key?: string): HceCodec
  withTimestampConfig(epochMs: number, gran: string): HceCodec
  withDomainModulus(n: string): HceCodec
}
```

## License

MIT
