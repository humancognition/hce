# HCE

Lossless, pronounceable, encrypted codec for any identifier — reversible, format-preserving, human-readable.

## Install

```bash
pip install hce
```

## Quick Start

```python
import hce

h = hce.Hce(
    key=b"32-byte-secret-key-here-xxxxxx!!",
    level="universal",
    mode="sealed",
)

encoded = h.encode(bytes(range(16)))
decoded = h.decode(encoded)
print(decoded.hex())
```

## API

### Constructor

```python
hce.Hce(
    key: bytes | None = None,
    level: str = "universal",
    mode: str = "sealed",
    bit_width: int = 128,
) -> Hce
```

| Param | Type | Description |
|-------|------|-------------|
| `key` | `bytes \| None` | Encryption key bytes. `None` for plain mode. |
| `level` | `str` | `"universal"`, `"eu"`, `"en"`, `"numeric"` |
| `mode` | `str` | `"sealed"`, `"open"`, `"plain"` |
| `bit_width` | `int` | `16`–`128`. Default `128`. |

### Core Methods

```python
h.encode(data: bytes) -> str
h.decode(input: str) -> bytes
h.recover(input: str) -> bytes
```

| Method | Description |
|--------|-------------|
| `encode(data)` | Encrypt + encode bytes to HCE string. |
| `decode(input)` | Decode + decrypt HCE string to bytes. Raises `ValueError` on integrity failure. |
| `recover(input)` | Attempt self-recovery on corrupted input. Returns recovered bytes or raises `ValueError`. |

### Domain Configuration

```python
h.with_bit_width(bits: int) -> Hce
h.with_modulus(modulus: int) -> Hce
h.with_cipher(kind: str, key: bytes | None = None) -> Hce
```

| Method | Description |
|--------|-------------|
| `with_bit_width(bits)` | Set cipher domain bit width (16-128). |
| `with_modulus(n)` | Set modulus domain. Must be ≥ 2. |
| `with_cipher(kind, key?)` | Select cipher: `"feistel"` (default, 8-round HMAC-SHA256) or `"shuffle"` (4-round lightweight). |

### Output Configuration

```python
h.with_case("upper" | "lower") -> Hce
h.with_check_syllables(n: int) -> Hce
h.with_separator(sep: str) -> Hce
```

| Method | Description |
|--------|-------------|
| `with_case(case)` | Output case. Default `"upper"`. |
| `with_check_syllables(n)` | Number of HMAC check syllables (1-8). Default `1`. |
| `with_separator(sep)` | Chunk separator character. Default `"-"`. |

### Chunking

```python
h.with_chunk_none() -> Hce
h.with_chunk_fixed(char_size: int) -> Hce
h.with_chunk_pattern(pattern: list[int]) -> Hce
```

| Method | Description |
|--------|-------------|
| `with_chunk_none()` | No separators. |
| `with_chunk_fixed(n)` | Chunk every `n` characters. |
| `with_chunk_pattern([3,3,4,4])` | Custom syllable-per-chunk pattern. |

### Timestamp

```python
h.with_timestamp_config(epoch_ms: int, granularity: str) -> Hce
```

| Param | Description |
|-------|-------------|
| `epoch_ms` | Unix epoch in milliseconds. |
| `granularity` | `"second" \| "minute" \| "hour" \| "day" \| "week" \| "month"` |

## Levels

Each level uses a different phoneme set:

| Level | Onset | Vowel+Nucleus | Description |
|-------|-------|---------------|-------------|
| `universal` | 29 | 20 | Cross-lingual, PHOIBLE 40% threshold |
| `eu` | 27 | 20 | European languages |
| `en` | 24 | 18 | English-specific |
| `numeric` | 10 | 5 | Numeric-style (0-9 mapped to phonemes) |

## Modes

| Mode | Check | Purpose |
|------|-------|---------|
| `sealed` | HMAC-SHA256 | Full confidentiality + integrity |
| `open` | HMAC-SHA256 | Timestamp prefix for sorting |
| `plain` | CRC | No encryption, CRC integrity only |

## Development

```bash
cd crates/hce-py
python -m venv .venv
source .venv/bin/activate
pip install maturin pytest
maturin develop
pytest tests/
```

## License

MIT
