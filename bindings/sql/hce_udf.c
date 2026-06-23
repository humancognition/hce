#include <mysql.h>
#include <string.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <dlfcn.h>

typedef struct RawHce RawHce;
typedef struct { bool ok; uint8_t *data; size_t len; int32_t err_code; } HceResult;

typedef RawHce *(*fn_new)(const uint8_t *, size_t, uint8_t, uint8_t);
typedef void (*fn_destroy)(RawHce *);
typedef HceResult (*fn_encode)(RawHce *, const uint8_t *, size_t);
typedef HceResult (*fn_decode)(RawHce *, const uint8_t *, size_t);
typedef void (*fn_free)(HceResult);

static fn_new    p_hce_new;
static fn_destroy p_hce_destroy;
static fn_encode  p_hce_encode;
static fn_decode  p_hce_decode;
static fn_free    p_hce_free_result;
static int lib_loaded = 0;

static void load_lib(void) {
    if (lib_loaded) return;
    void *h = dlopen("libhce_ffi.so", RTLD_NOW);
    if (!h) return;
    p_hce_new       = (fn_new)dlsym(h, "hce_new");
    p_hce_destroy   = (fn_destroy)dlsym(h, "hce_destroy");
    p_hce_encode    = (fn_encode)dlsym(h, "hce_encode");
    p_hce_decode    = (fn_decode)dlsym(h, "hce_decode");
    p_hce_free_result = (fn_free)dlsym(h, "hce_free_result");
    lib_loaded = 1;
}

static uint8_t default_key[32] = "hce-kat-standard-key-32-bytes!!";

static int parse_args(UDF_ARGS *args, uint8_t **key, size_t *key_len, uint8_t *level, uint8_t *mode) {
    *key = default_key;
    *key_len = 32;
    *level = 0;
    *mode = 0;
    if (args->arg_count < 1 || args->arg_type[0] != STRING_RESULT) return 0;
    if (args->arg_count >= 2 && args->arg_type[1] == STRING_RESULT && args->lengths[1] > 0) {
        *key = (uint8_t *)args->args[1];
        *key_len = args->lengths[1];
    }
    if (args->arg_count >= 3 && args->arg_type[2] == INT_RESULT)
        *level = (uint8_t)(*((long long *)args->args[2]));
    if (args->arg_count >= 4 && args->arg_type[3] == INT_RESULT)
        *mode = (uint8_t)(*((long long *)args->args[3]));
    return 1;
}

bool hce_encode_init(UDF_INIT *initid, UDF_ARGS *args, char *message) {
    if (args->arg_count < 1 || args->arg_count > 4 || args->arg_type[0] != STRING_RESULT) {
        strcpy(message, "hce_encode(data [, key, level, mode])");
        return 1;
    }
    load_lib();
    if (!p_hce_encode) { strcpy(message, "failed to load libhce_ffi.so"); return 1; }
    initid->max_length = 256;
    initid->maybe_null = 1;
    return 0;
}

char *hce_encode(UDF_INIT *initid, UDF_ARGS *args, char *result,
                 unsigned long *length, unsigned char *is_null, unsigned char *error) {
    uint8_t *key; size_t key_len; uint8_t level, mode;
    if (!parse_args(args, &key, &key_len, &level, &mode)) { *error = 1; return NULL; }
    RawHce *h = p_hce_new(key, key_len, level, mode);
    if (!h) { *error = 1; return NULL; }
    HceResult r = p_hce_encode(h, (const uint8_t *)args->args[0], args->lengths[0]);
    p_hce_destroy(h);
    if (!r.ok) { p_hce_free_result(r); *error = 1; return NULL; }
    char *out = malloc(r.len + 1);
    if (!out) { p_hce_free_result(r); *error = 1; return NULL; }
    memcpy(out, r.data, r.len); out[r.len] = '\0';
    *length = r.len;
    p_hce_free_result(r);
    return out;
}

void hce_encode_deinit(UDF_INIT *initid) {}

bool hce_decode_init(UDF_INIT *initid, UDF_ARGS *args, char *message) {
    if (args->arg_count < 1 || args->arg_count > 4 || args->arg_type[0] != STRING_RESULT) {
        strcpy(message, "hce_decode(hce_str [, key, level, mode])");
        return 1;
    }
    load_lib();
    if (!p_hce_decode) { strcpy(message, "failed to load libhce_ffi.so"); return 1; }
    initid->max_length = 16;
    initid->maybe_null = 1;
    return 0;
}

char *hce_decode(UDF_INIT *initid, UDF_ARGS *args, char *result,
                 unsigned long *length, unsigned char *is_null, unsigned char *error) {
    uint8_t *key; size_t key_len; uint8_t level, mode;
    if (!parse_args(args, &key, &key_len, &level, &mode)) { *error = 1; return NULL; }
    RawHce *h = p_hce_new(key, key_len, level, mode);
    if (!h) { *error = 1; return NULL; }
    HceResult r = p_hce_decode(h, (const uint8_t *)args->args[0], args->lengths[0]);
    p_hce_destroy(h);
    if (!r.ok) { p_hce_free_result(r); *error = 1; return NULL; }
    char *out = malloc(r.len);
    if (!out) { p_hce_free_result(r); *error = 1; return NULL; }
    memcpy(out, r.data, r.len);
    *length = r.len;
    p_hce_free_result(r);
    return out;
}

void hce_decode_deinit(UDF_INIT *initid) {}
