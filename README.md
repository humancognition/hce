# HCE

Lossless, pronounceable, encrypted codec for any identifier — reversible, format-preserving, human-readable.

- **Encrypted.** HMAC-SHA256 Feistel FPE (8 rounds). Custom cipher injection.
- **Pronounceable.** 35 phonemic onsets × 20 vowel nuclei across 4 languages.
- **Self-recovering.** Corrects common phoneme confusions.
- **Format-preserving.** 128-bit input → ~50 char output. Configurable chunking.
- **Multi-platform.** Rust, C, WASM, Python, Go, Java, Swift, Elixir, SQL.

```rust
use hce_core::{Hce, HceMode, LanguageLevel};

let hce = Hce::new(
    Some(b"32-byte-secret-key-here-xxxxxx!!"),
    LanguageLevel::Universal,
    HceMode::Sealed,
);

let id = hce.encode(&uuid_bytes);
//=> "PETREN-NISLORPEN-LAFLER-SRORGULGOLFUN-PREPLEN"

let bytes = hce.decode(&id).unwrap();
```

## Packages

| Package | Language | Description |
|---------|----------|-------------|
| `hce-core` | Rust | Core library. |
| `hce-fpe` | Rust | Cipher factory. Feistel, Shuffle, future FF3-1. |
| `hce-adapters` | Rust | UUID, ULID, ObjectId, Snowflake, Xid. |
| `hce-cli` | Rust | CLI: `hce encode`, `hce decode`, `hce recover`. |
| `hce-ffi` | C | C ABI. Header via cbindgen. |
| `hce-wasm` | WASM | npm package `@humancognition/hce`. |
| `hce-py` | Python | PyPI package `hce`. |
| `bindings/go` | Go | cgo wrapper. |
| `bindings/java` | Java | JNA + Maven. |
| `bindings/swift` | Swift | C interop + SPM. |
| `bindings/elixir` | Elixir | Rustler NIF. |
| `bindings/sql` | SQL | MySQL UDF. |

## Install

```toml
[dependencies]
hce-core = "0.1"
```

```bash
npm install hce
pip install hce
cargo install hce-cli
```

## API

### Constructor

```rust
Hce::new(key: Option<&[u8]>, level: LanguageLevel, mode: HceMode) -> Hce
```

| Param | Description |
|-------|-------------|
| `key` | Encryption key. `None` for plain mode. |
| `level` | `Universal` \| `Eu` \| `En` \| `Numeric` |
| `mode` | `Sealed` \| `Open` \| `Plain` |

### Core

```rust
hce.encode(&[u8]) -> String
hce.decode(&str) -> Result<Vec<u8>, HceError>
hce.recover(&str) -> Result<RecoveryResult, HceError>
```

### Configuration

```rust
hce.with_bit_width(u32)       // 16–128, default 128
hce.with_modulus(u128)        // modulus ≥ 2
hce.with_case(HceCase)        // Upper | Lower
hce.with_check_syllables(usize) // 1–8, default 1
hce.with_separator(char)      // default '-'
hce.with_chunk_none()
hce.with_chunk_fixed(usize)
hce.with_chunk_pattern(&[usize])
hce.with_timestamp_config(i64, TsGranularity)
hce.with_cipher(Arc<dyn Fpe>) // custom FPE
```

## Levels

| Level | Onsets | Vowel+Nucleus | Radix | Description |
|-------|--------|---------------|-------|-------------|
| `Universal` | 35 | 20 | 700 | Cross-lingual, PHOIBLE 40% threshold |
| `Eu` | 35 | 20 | 700 | European languages |
| `En` | 35 | 20 | 700 | English-specific |
| `Numeric` | 35 | 20 | 700 | Numeric-style (text representation) |

## Modes

| Mode | Check | Timestamp | Purpose |
|------|-------|-----------|---------|
| `Sealed` | HMAC-SHA256 | No | Full confidentiality + integrity |
| `Open` | HMAC-SHA256 | Yes | Sortable, timestamp-prefixed |
| `Plain` | CRC | No | No encryption, CRC integrity only |

## Other Languages

- [C](crates/hce-ffi/README.md)
- [Elixir](bindings/elixir/README.md)
- [Go](bindings/go/README.md)
- [Java](bindings/java/README.md)
- [MySQL/MariaDB](bindings/sql/README.md)
- [Node.js](bindings/npm/hce/README.md)
- [Python](crates/hce-py/README.md)
- [Rust](crates/hce-core/README.md)
- [Swift](bindings/swift/README.md)

## License

MIT
