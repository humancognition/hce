# HCE — Swift

Lossless, pronounceable, encrypted codec for any identifier.

## Install

Add to `Package.swift`:

```swift
dependencies: [
    .package(path: "bindings/swift"),
],
targets: [
    .target(name: "MyApp", dependencies: ["Hce"]),
]
```

Requires `libhce_ffi.dylib` built:

```bash
cargo build -p hce-ffi --release
cp target/release/libhce_ffi.dylib /usr/local/lib/
```

## Usage

```swift
import Hce

let key = "32-byte-secret-key-here-xxxxxx!!".data(using: .utf8)!
let uuid = Data([1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39])

guard let h = HceCodec(key: key, level: .universal, mode: .sealed) else { return }

if let encoded = h.encode(uuid) {
    print(encoded)
}
if let decoded = h.decode(encoded) {
    print(decoded.hexString)
}
```

## API

```swift
HceCodec(key: Data?, level: Level, mode: Mode)

func encode(Data) -> String?
func decode(String) -> Data?
func recover(String) -> Data?

func withBitWidth(UInt32) -> HceCodec
func withModulus(UInt64) -> HceCodec
func withCipherFeistel(Data?) -> HceCodec
func withCipherShuffle() -> HceCodec
func withCase(CipherCase) -> HceCodec
func withCheckSyllables(Int) -> HceCodec
func withSeparator(Character) -> HceCodec
func withChunkNone() -> HceCodec
func withChunkFixed(Int) -> HceCodec
func withChunkPattern([Int]) -> HceCodec
func withTimestampConfig(Int64, Granularity) -> HceCodec
```

## License

MIT
