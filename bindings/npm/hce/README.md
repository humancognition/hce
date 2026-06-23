# hce

Lossless, pronounceable, encrypted codec for any identifier.

```js
import { randomBytes } from "crypto";

const key = Buffer.from("32-byte-secret-key-here-xxxxxx!!").toString("hex");
const hce = new HceCodec(key, "universal", "sealed");

const id = hce.encode(randomBytes(16));
//=> "PETREN-NISLORPEN-LAFLER-SRORGULGOLFUN-PREPLEN"

const uuid = hce.decode(id);
```

## Install

```bash
npm install @humancognition/hce
```

## API

```ts
new HceCodec(key: string | null, level: string, mode: string): HceCodec
hce.encode(data: Uint8Array): string
hce.decode(input: string): Uint8Array
hce.recover(input: string): Uint8Array
```

### Configuration

```ts
hce.withBitWidth(32 | 64 | 96 | 128)       // default 128
hce.withDomainModulus(n: string)             // decimal string
hce.withCase(uppercase: boolean)             // default true
hce.withCheckSyllables(n: number)            // default 1
hce.withSeparator(sep: string)               // default "-"
hce.withChunkNone()
hce.withChunkFixed(charSize: number)
hce.withChunkPattern(pattern: number[])
hce.withCipher("feistel" | "shuffle", key?: string)
hce.withTimestampConfig(epochMs: number, gran: string)
```

## Node.js

```js
import { readFileSync } from "fs";
import { initSync, HceCodec } from "@humancognition/hce";

const wasm = readFileSync("node_modules/@humancognition/hce/index_bg.wasm");
initSync(wasm);
```

## Browser

```js
import init, { HceCodec } from "@humancognition/hce";
await init();
```

## License

MIT
