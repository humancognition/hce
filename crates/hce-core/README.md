# hce-core

Lossless, pronounceable, encrypted codec for any identifier — reversible, format-preserving, human-readable.

Only `hmac` + `sha2`+ `zeroing` dependencies.

## Usage

```rust
use hce_core::{Hce, HceMode, LanguageLevel};

let hce = Hce::new(
    Some(b"32-byte-secret-key-here-xxxxxx!!"),
    LanguageLevel::Universal,
    HceMode::Sealed,
);

let id = hce.encode(&[1, 149, 227, 160, 124, 46, 123, 65,
                       143, 61, 154, 108, 30, 11, 77, 39]);

let bytes = hce.decode(&id).unwrap();
assert_eq!(bytes, vec![1, 149, 227, 160, 124, 46, 123, 65,
                        143, 61, 154, 108, 30, 11, 77, 39]);
```

## API

| Method | Description |
|--------|-------------|
| `Hce::new(key, level, mode)` | Create a codec |
| `encode(&[u8]) -> String` | Encode bytes to HCE string |
| `decode(&str) -> Result<Vec<u8>>` | Decode HCE string to bytes |
| `recover(&str) -> Result<RecoveryResult>` | Attempt self-recovery |
| `with_bit_width(u32)` | Set domain bit width |
| `with_modulus(u128)` | Set modulus domain |
| `with_cipher(Arc<dyn Fpe>)` | Inject custom FPE |
| `with_case(HceCase)` | Upper/lower case |
| `with_chunk_spec(ChunkSpec)` | Chunk configuration |
| `with_timestamp_config(i64, TsGranularity)` | Timestamp prefix |

## License

MIT
