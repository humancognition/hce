# HCE — C / FFI

Lossless, pronounceable, encrypted codec for any identifier — reversible, format-preserving, human-readable.

## Install

```bash
cargo build -p hce-ffi --release
```

| OS | Library | 
|----|---------|
| macOS | `target/release/libhce_ffi.dylib` |
| Linux | `target/release/libhce_ffi.so` |
| Windows | `target/release/hce_ffi.dll` |

```bash
cbindgen --config crates/hce-ffi/cbindgen.toml --crate hce-ffi --output hce.h
```

## Quick Start

```c
#include "hce.h"

int main(void) {
    uint8_t key[32] = "32-byte-secret-key-here-xxxxxx!!";
    uint8_t uuid[16] = {1,149,227,160,124,46,123,65,143,61,154,108,30,11,77,39};

    RawHce *h = hce_new(key, 32, HCE_LEVEL_UNIVERSAL, HCE_MODE_SEALED);

    HceResult enc = hce_encode(h, uuid, 16);
    uint8_t *output = enc.data; /* enc.len bytes, owned */

    HceResult dec = hce_decode(h, output, enc.len);
    uint8_t *original = dec.data; /* 16 bytes */

    hce_free_result(enc);
    hce_free_result(dec);
    hce_destroy(h);
    return 0;
}
```

Build:

```bash
cc -I crates/hce-ffi -L target/release -o myapp myapp.c -lhce_ffi
DYLD_LIBRARY_PATH=target/release ./myapp
```

## API Reference

### Lifecycle

```c
RawHce *hce_new(const uint8_t *key, size_t key_len, uint8_t level, uint8_t mode);
RawHce *hce_new_with_cipher(const FpeVtable *vtable, uint8_t level, uint8_t mode);
void     hce_destroy(RawHce *h);
RawHce *hce_clone(const RawHce *h);
```

### Encode / Decode / Recover

```c
HceResult        hce_encode(const RawHce *h, const uint8_t *data, size_t len);
HceResult        hce_decode(const RawHce *h, const uint8_t *input, size_t len);
RecoveryCResult  hce_recover(const RawHce *h, const uint8_t *input, size_t len);
```

### Result Types

```c
typedef struct {
    bool ok;
    uint8_t *data;      /* owned, free with hce_free_result */
    size_t len;
    int32_t err_code;
} HceResult;

typedef struct {
    bool ok;
    uint8_t *corrected; /* owned, free with hce_free_recovery_result */
    size_t corrected_len;
    size_t candidate_count;
    int32_t err_code;
} RecoveryCResult;
```

### Cleanup

```c
void hce_free_result(HceResult result);
void hce_free_recovery_result(RecoveryCResult result);
void hce_free_buf(uint8_t *ptr, size_t len);
const uint8_t *hce_error_string(int32_t code);
```

### Domain Configuration

```c
void hce_with_bit_width(RawHce *h, uint32_t bits);
void hce_with_domain_modulus(RawHce *h, uint64_t hi, uint64_t lo);
void hce_with_cipher_kind(RawHce *h, uint8_t kind, const uint8_t *key, size_t key_len);
```

| `kind` | Cipher |
|--------|--------|
| `0` | Feistel (8-round HMAC-SHA256) |
| `1` | Shuffle (4-round lightweight) |

### Output Configuration

```c
void hce_with_case(RawHce *h, uint8_t case_val);
void hce_with_check_syllables(RawHce *h, size_t n);
void hce_with_separator(RawHce *h, uint8_t sep);
```

### Chunking

```c
void hce_with_chunk_none(RawHce *h);
void hce_with_chunk_fixed(RawHce *h, size_t char_size);
void hce_with_chunk_pattern(RawHce *h, const size_t *pattern, size_t count);
```

### Timestamp

```c
void hce_with_timestamp_config(RawHce *h, int64_t epoch_ms, uint8_t granularity);
```

### Constants

```c
/* Levels */
HCE_LEVEL_UNIVERSAL  HCE_LEVEL_EU  HCE_LEVEL_EN  HCE_LEVEL_NUMERIC

/* Modes */
HCE_MODE_SEALED  HCE_MODE_OPEN  HCE_MODE_PLAIN

/* Case */
HCE_CASE_LOWER  HCE_CASE_UPPER

/* Granularity */
HCE_GRANULARITY_SECOND  HCE_GRANULARITY_MINUTE  HCE_GRANULARITY_HOUR
HCE_GRANULARITY_DAY     HCE_GRANULARITY_WEEK    HCE_GRANULARITY_MONTH

/* Error codes */
HCE_ERR_NORMALIZE  HCE_ERR_KEY_REQUIRED  HCE_ERR_RECOVERY_NOT_SUPPORTED
HCE_ERR_INTEGRITY  HCE_ERR_NULL_PTR
```

## Custom Cipher (FF3-1)

```c
void my_ff3_encrypt(void *ctx, const uint8_t *plain, const uint8_t *tweak, uint8_t *out) {
    ff3_encrypt((MyFF3State *)ctx, plain, tweak, out);
}
void my_ff3_decrypt(void *ctx, const uint8_t *cipher, const uint8_t *tweak, uint8_t *out) {
    ff3_decrypt((MyFF3State *)ctx, cipher, tweak, out);
}

MyFF3State state;
ff3_init(&state, "2DE79D232DF5585D68CE47882AE256D6", 10);

FpeVtable vt = {
    .context = &state,
    .encrypt = my_ff3_encrypt,
    .decrypt = my_ff3_decrypt,
};
RawHce *h = hce_new_with_cipher(&vt, HCE_LEVEL_UNIVERSAL, HCE_MODE_SEALED);
```

## License

MIT
