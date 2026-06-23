#ifndef HCE_FFI_H
#define HCE_FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#define HCE_LEVEL_UNIVERSAL   0
#define HCE_LEVEL_EU          1
#define HCE_LEVEL_EN          2
#define HCE_LEVEL_NUMERIC     3
#define HCE_MODE_SEALED       0
#define HCE_MODE_OPEN         1
#define HCE_MODE_PLAIN        2
#define HCE_CASE_LOWER        0
#define HCE_CASE_UPPER        1
#define HCE_GRANULARITY_SECOND  0
#define HCE_GRANULARITY_MINUTE  1
#define HCE_GRANULARITY_HOUR    2
#define HCE_GRANULARITY_DAY     3
#define HCE_GRANULARITY_WEEK    4
#define HCE_GRANULARITY_MONTH   5
#define HCE_ERR_NORMALIZE              1
#define HCE_ERR_KEY_REQUIRED           2
#define HCE_ERR_RECOVERY_NOT_SUPPORTED  3
#define HCE_ERR_INTEGRITY              4
#define HCE_ERR_NULL_PTR               5

typedef struct RawHce RawHce;

typedef struct FpeVtable {
  void *context;
  void (*encrypt)(void *ctx, const uint8_t *plain, const uint8_t *tweak, uint8_t *cipher_out);
  void (*decrypt)(void *ctx, const uint8_t *cipher, const uint8_t *tweak, uint8_t *plain_out);
} FpeVtable;

typedef struct HceResult {
  bool ok;
  uint8_t *data;
  size_t len;
  int32_t err_code;
} HceResult;

typedef struct RecoveryCResult {
  bool ok;
  uint8_t *corrected;
  size_t corrected_len;
  size_t candidate_count;
  int32_t err_code;
} RecoveryCResult;

struct RawHce *hce_new(const uint8_t *key_ptr, size_t key_len, uint8_t level, uint8_t mode);

struct RawHce *hce_new_with_cipher(const struct FpeVtable *vtable, uint8_t level, uint8_t mode);

void hce_destroy(struct RawHce *hce);

struct RawHce *hce_clone(const struct RawHce *hce);

void hce_with_bit_width(struct RawHce *hce, uint32_t bits);

void hce_with_case(struct RawHce *hce, uint8_t case_);

void hce_with_check_syllables(struct RawHce *hce, size_t n);

void hce_with_separator(struct RawHce *hce, uint8_t sep);

void hce_with_chunk_none(struct RawHce *hce);

void hce_with_chunk_fixed(struct RawHce *hce, size_t char_size);

void hce_with_chunk_pattern(struct RawHce *hce, const size_t *pattern, size_t count);

void hce_with_timestamp_config(struct RawHce *hce, int64_t epoch_ms, uint8_t granularity);

void hce_with_domain_modulus(struct RawHce *hce, uint64_t modulus_hi, uint64_t modulus_lo);

void hce_with_cipher_kind(struct RawHce *hce, uint8_t kind, const uint8_t *key_ptr, size_t key_len);

struct HceResult hce_encode(const struct RawHce *hce, const uint8_t *data_ptr, size_t data_len);

struct HceResult hce_decode(const struct RawHce *hce, const uint8_t *input_ptr, size_t input_len);

struct RecoveryCResult hce_recover(const struct RawHce *hce,
                                   const uint8_t *input_ptr,
                                   size_t input_len);

void hce_free_result(struct HceResult result);

void hce_free_recovery_result(struct RecoveryCResult result);

const uint8_t *hce_error_string(int32_t code);

struct HceResult hce_adapter_encode(uint8_t adapter_kind,
                                    const uint8_t *key_ptr,
                                    size_t key_len,
                                    const uint8_t *id_ptr,
                                    size_t id_len);

void hce_free_buf(uint8_t *ptr, size_t len);

#endif  /* HCE_FFI_H */
