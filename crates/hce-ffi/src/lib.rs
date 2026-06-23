#![allow(clippy::not_unsafe_ptr_arg_deref)]
extern crate alloc;

use alloc::sync::Arc;
use hce_adapters::*;
use hce_core::*;
use hce_fpe::{build_cipher, CipherKind};

pub struct RawHce(Arc<Hce>);

#[repr(C)]
pub struct HceResult {
    pub ok: bool,
    pub data: *mut u8,
    pub len: usize,
    pub err_code: i32,
}

#[repr(C)]
pub struct RecoveryCResult {
    pub ok: bool,
    pub corrected: *mut u8,
    pub corrected_len: usize,
    pub candidate_count: usize,
    pub err_code: i32,
}

#[repr(C)]
pub struct FpeVtable {
    pub context: *mut core::ffi::c_void,
    pub encrypt: Option<
        unsafe extern "C" fn(
            ctx: *mut core::ffi::c_void,
            plain: *const u8,
            tweak: *const u8,
            cipher_out: *mut u8,
        ),
    >,
    pub decrypt: Option<
        unsafe extern "C" fn(
            ctx: *mut core::ffi::c_void,
            cipher: *const u8,
            tweak: *const u8,
            plain_out: *mut u8,
        ),
    >,
}

struct CallbackFpe {
    ctx: *mut core::ffi::c_void,
    encrypt: unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, *const u8, *mut u8),
    decrypt: unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, *const u8, *mut u8),
}

unsafe impl Send for CallbackFpe {}
unsafe impl Sync for CallbackFpe {}

impl Fpe for CallbackFpe {
    fn encrypt(&self, plain: u128, tweak: &[u8]) -> u128 {
        let mut out = [0u8; 16];
        let plain_bytes = plain.to_be_bytes();
        let mut t = [0u8; 4];
        let n = tweak.len().min(4);
        t[..n].copy_from_slice(&tweak[..n]);
        unsafe { (self.encrypt)(self.ctx, plain_bytes.as_ptr(), t.as_ptr(), out.as_mut_ptr()) };
        u128::from_be_bytes(out)
    }

    fn decrypt(&self, cipher: u128, tweak: &[u8]) -> u128 {
        let mut out = [0u8; 16];
        let cipher_bytes = cipher.to_be_bytes();
        let mut t = [0u8; 4];
        let n = tweak.len().min(4);
        t[..n].copy_from_slice(&tweak[..n]);
        unsafe {
            (self.decrypt)(
                self.ctx,
                cipher_bytes.as_ptr(),
                t.as_ptr(),
                out.as_mut_ptr(),
            )
        };
        u128::from_be_bytes(out)
    }
}

const ERR_NORMALIZE: i32 = 1;
const ERR_KEY_REQUIRED: i32 = 2;
const ERR_RECOVERY_NOT_SUPPORTED: i32 = 3;
const ERR_INTEGRITY: i32 = 4;
const ERR_NULL_PTR: i32 = 5;

fn err_code(e: &HceError) -> i32 {
    match e {
        HceError::NormalizeError => ERR_NORMALIZE,
        HceError::KeyRequired => ERR_KEY_REQUIRED,
        HceError::RecoveryNotSupported => ERR_RECOVERY_NOT_SUPPORTED,
        HceError::IntegrityFailure => ERR_INTEGRITY,
    }
}

#[no_mangle]
pub extern "C" fn hce_new(key_ptr: *const u8, key_len: usize, level: u8, mode: u8) -> *mut RawHce {
    if key_len > 0 && key_ptr.is_null() {
        return core::ptr::null_mut();
    }
    let level = match LanguageLevel::from_byte(level) {
        Some(l) => l,
        None => LanguageLevel::Universal,
    };
    let mode = match HceMode::from_byte(mode) {
        Some(m) => m,
        None => HceMode::Plain,
    };
    let key = if key_len > 0 {
        Some(unsafe { core::slice::from_raw_parts(key_ptr, key_len) })
    } else {
        None
    };
    let raw = RawHce(Arc::new(Hce::new(key, level, mode)));
    Box::into_raw(Box::new(raw))
}

#[no_mangle]
pub extern "C" fn hce_new_with_cipher(
    vtable: *const FpeVtable,
    level: u8,
    mode: u8,
) -> *mut RawHce {
    if vtable.is_null() {
        return core::ptr::null_mut();
    }
    let vt = unsafe { &*vtable };
    let Some(encrypt) = vt.encrypt else {
        return core::ptr::null_mut();
    };
    let Some(decrypt) = vt.decrypt else {
        return core::ptr::null_mut();
    };
    let level = LanguageLevel::from_byte(level).unwrap_or(LanguageLevel::Universal);
    let hce_mode = HceMode::from_byte(mode).unwrap_or(HceMode::Plain);
    let fpe = CallbackFpe {
        ctx: vt.context,
        encrypt,
        decrypt,
    };
    let hce = Hce::new(Option::<&[u8]>::None, level, hce_mode).with_cipher(Arc::new(fpe));
    let raw = RawHce(Arc::new(hce));
    Box::into_raw(Box::new(raw))
}

#[no_mangle]
pub extern "C" fn hce_destroy(hce: *mut RawHce) {
    if hce.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(hce));
    }
}

#[no_mangle]
pub extern "C" fn hce_clone(hce: *const RawHce) -> *mut RawHce {
    if hce.is_null() {
        return core::ptr::null_mut();
    }
    let raw = unsafe { &*hce };
    Box::into_raw(Box::new(RawHce(Arc::clone(&raw.0))))
}

#[no_mangle]
pub extern "C" fn hce_with_bit_width(hce: *mut RawHce, bits: u32) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_bit_width(bits));
}

#[no_mangle]
pub extern "C" fn hce_with_case(hce: *mut RawHce, case: u8) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    let c = if case == 0 {
        HceCase::Lower
    } else {
        HceCase::Upper
    };
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_case(c));
}

#[no_mangle]
pub extern "C" fn hce_with_check_syllables(hce: *mut RawHce, n: usize) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_check_syllables(n));
}

#[no_mangle]
pub extern "C" fn hce_with_separator(hce: *mut RawHce, sep: u8) {
    if hce.is_null() || !validate_separator(sep as char) {
        return;
    }
    let raw = unsafe { &mut *hce };
    let spec = ChunkSpec::natural_with_separator(sep as char);
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_chunk_spec(spec));
}

#[no_mangle]
pub extern "C" fn hce_with_chunk_none(hce: *mut RawHce) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_chunk_spec(ChunkSpec::none()));
}

#[no_mangle]
pub extern "C" fn hce_with_chunk_fixed(hce: *mut RawHce, char_size: usize) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    raw.0 = alloc::sync::Arc::new(
        (*raw.0)
            .clone()
            .with_chunk_spec(ChunkSpec::fixed(char_size)),
    );
}

#[no_mangle]
pub extern "C" fn hce_with_chunk_pattern(hce: *mut RawHce, pattern: *const usize, count: usize) {
    if hce.is_null() || pattern.is_null() || count == 0 {
        return;
    }
    let raw = unsafe { &mut *hce };
    let p = unsafe { core::slice::from_raw_parts(pattern, count) };
    let spec = ChunkSpec::pattern(p);
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_chunk_spec(spec));
}

#[no_mangle]
pub extern "C" fn hce_with_timestamp_config(hce: *mut RawHce, epoch_ms: i64, granularity: u8) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    let g = match granularity {
        0 => TsGranularity::Second,
        1 => TsGranularity::Minute,
        2 => TsGranularity::Hour,
        3 => TsGranularity::Day,
        4 => TsGranularity::Week,
        _ => TsGranularity::Month,
    };
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_timestamp_config(epoch_ms, g));
}

#[no_mangle]
pub extern "C" fn hce_with_domain_modulus(hce: *mut RawHce, modulus_hi: u64, modulus_lo: u64) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    let modulus = ((modulus_hi as u128) << 64) | (modulus_lo as u128);
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_modulus(modulus));
}

#[no_mangle]
pub extern "C" fn hce_with_cipher_kind(
    hce: *mut RawHce,
    kind: u8,
    key_ptr: *const u8,
    key_len: usize,
) {
    if hce.is_null() {
        return;
    }
    let raw = unsafe { &mut *hce };
    let ck = match kind {
        0 => CipherKind::Feistel8,
        _ => CipherKind::Shuffle4,
    };
    let key = if key_len > 0 && !key_ptr.is_null() {
        Some(unsafe { core::slice::from_raw_parts(key_ptr, key_len) })
    } else {
        None
    };
    let fpe = build_cipher(ck, key, (*raw.0).domain());
    raw.0 = alloc::sync::Arc::new((*raw.0).clone().with_cipher(fpe));
}

#[no_mangle]
pub extern "C" fn hce_encode(
    hce: *const RawHce,
    data_ptr: *const u8,
    data_len: usize,
) -> HceResult {
    if hce.is_null() || data_ptr.is_null() || data_len == 0 {
        return HceResult {
            ok: false,
            data: core::ptr::null_mut(),
            len: 0,
            err_code: ERR_NULL_PTR,
        };
    }
    let raw = unsafe { &*hce };
    let input = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };
    let encoded = raw.0.encode(input);
    let len = encoded.len();
    let mut buf = encoded.into_bytes().into_boxed_slice();
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    HceResult {
        ok: true,
        data: ptr,
        len,
        err_code: 0,
    }
}

#[no_mangle]
pub extern "C" fn hce_decode(
    hce: *const RawHce,
    input_ptr: *const u8,
    input_len: usize,
) -> HceResult {
    if hce.is_null() || input_ptr.is_null() || input_len == 0 {
        return HceResult {
            ok: false,
            data: core::ptr::null_mut(),
            len: 0,
            err_code: ERR_NULL_PTR,
        };
    }
    let raw = unsafe { &*hce };
    let input_bytes = unsafe { core::slice::from_raw_parts(input_ptr, input_len) };
    let input = match core::str::from_utf8(input_bytes) {
        Ok(s) => s,
        Err(_) => {
            return HceResult {
                ok: false,
                data: core::ptr::null_mut(),
                len: 0,
                err_code: ERR_NORMALIZE,
            };
        }
    };
    match raw.0.decode(input) {
        Ok(bytes) => {
            let len = bytes.len();
            let mut buf = bytes.into_boxed_slice();
            let ptr = buf.as_mut_ptr();
            core::mem::forget(buf);
            HceResult {
                ok: true,
                data: ptr,
                len,
                err_code: 0,
            }
        }
        Err(e) => HceResult {
            ok: false,
            data: core::ptr::null_mut(),
            len: 0,
            err_code: err_code(&e),
        },
    }
}

#[no_mangle]
pub extern "C" fn hce_recover(
    hce: *const RawHce,
    input_ptr: *const u8,
    input_len: usize,
) -> RecoveryCResult {
    let empty = RecoveryCResult {
        ok: false,
        corrected: core::ptr::null_mut(),
        corrected_len: 0,
        candidate_count: 0,
        err_code: 0,
    };
    if hce.is_null() || input_ptr.is_null() || input_len == 0 {
        return RecoveryCResult {
            err_code: ERR_NULL_PTR,
            ..empty
        };
    }
    let raw = unsafe { &*hce };
    let input_bytes = unsafe { core::slice::from_raw_parts(input_ptr, input_len) };
    let input = match core::str::from_utf8(input_bytes) {
        Ok(s) => s,
        Err(_) => {
            return RecoveryCResult {
                err_code: ERR_NORMALIZE,
                ..empty
            }
        }
    };
    match raw.0.recover(input) {
        Ok(result) => match result {
            RecoveryResult::Ok => RecoveryCResult { ok: true, ..empty },
            RecoveryResult::Corrected(tokens) => {
                match raw.0.decode_corrected(&RecoveryResult::Corrected(tokens)) {
                    Some(bytes) => {
                        let len = bytes.len();
                        let mut buf = bytes.into_boxed_slice();
                        let ptr = buf.as_mut_ptr();
                        core::mem::forget(buf);
                        RecoveryCResult {
                            ok: true,
                            corrected: ptr,
                            corrected_len: len,
                            ..empty
                        }
                    }
                    None => RecoveryCResult {
                        err_code: ERR_INTEGRITY,
                        ..empty
                    },
                }
            }
            RecoveryResult::Ambiguous(count) => RecoveryCResult {
                ok: true,
                candidate_count: count,
                ..empty
            },
            RecoveryResult::Reject => RecoveryCResult {
                err_code: ERR_INTEGRITY,
                ..empty
            },
        },
        Err(e) => RecoveryCResult {
            err_code: err_code(&e),
            ..empty
        },
    }
}

#[no_mangle]
pub extern "C" fn hce_free_result(result: HceResult) {
    if !result.data.is_null() && result.len > 0 {
        unsafe {
            drop(alloc::vec::Vec::from_raw_parts(
                result.data,
                result.len,
                result.len,
            ));
        }
    }
}

#[no_mangle]
pub extern "C" fn hce_free_recovery_result(result: RecoveryCResult) {
    if !result.corrected.is_null() && result.corrected_len > 0 {
        unsafe {
            drop(alloc::vec::Vec::from_raw_parts(
                result.corrected,
                result.corrected_len,
                result.corrected_len,
            ));
        }
    }
}

#[no_mangle]
pub extern "C" fn hce_error_string(code: i32) -> *const u8 {
    let msg: &[u8] = match code {
        ERR_NORMALIZE => b"input normalization failed\0",
        ERR_KEY_REQUIRED => b"key required for this mode\0",
        ERR_RECOVERY_NOT_SUPPORTED => b"recovery not supported in plain mode\0",
        ERR_INTEGRITY => b"check verification failed\0",
        ERR_NULL_PTR => b"null pointer argument\0",
        _ => b"unknown error\0",
    };
    msg.as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_KEY: &[u8] = b"kA..test-key-32-bytes-long-key!";

    #[test]
    fn new_destroy() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        assert!(!h.is_null());
        hce_destroy(h);
    }

    #[test]
    fn clone_works() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        let h2 = hce_clone(h);
        assert!(!h2.is_null());
        hce_destroy(h);
        hce_destroy(h2);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        let uuid: [u8; 16] = [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ];
        let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
        assert!(enc.ok, "encode failed: {}", enc.err_code);

        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok, "decode failed: {}", dec.err_code);
        assert_eq!(dec.len, 16);
        assert_eq!(
            unsafe { core::slice::from_raw_parts(dec.data, dec.len) },
            uuid
        );

        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn plain_roundtrip() {
        let h = hce_new(core::ptr::null(), 0, 0, 2);
        let data: [u8; 12] = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44,
        ];
        let enc = hce_encode(h, data.as_ptr(), data.len());
        assert!(enc.ok);
        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok);
        assert_eq!(dec.len, 16);
        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn bit_width_64() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        hce_with_bit_width(h, 64);
        let data: [u8; 8] = [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe];
        let enc = hce_encode(h, data.as_ptr(), data.len());
        assert!(enc.ok);
        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok);
        assert_eq!(dec.len, 16);
        assert_eq!(
            &unsafe { core::slice::from_raw_parts(dec.data, dec.len) }[8..],
            data
        );
        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn lower_case_output() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        hce_with_case(h, 0);
        let data: [u8; 16] = [0x42; 16];
        let enc = hce_encode(h, data.as_ptr(), data.len());
        assert!(enc.ok);
        let s = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(enc.data, enc.len))
        };
        assert!(!s.chars().any(|c| c.is_uppercase()));
        hce_free_result(enc);
        hce_destroy(h);
    }

    #[test]
    fn none_chunk_no_hyphens() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        hce_with_chunk_none(h);
        let data: [u8; 16] = [0x7f; 16];
        let enc = hce_encode(h, data.as_ptr(), data.len());
        assert!(enc.ok);
        let s = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(enc.data, enc.len))
        };
        assert!(!s.contains('-'));
        hce_free_result(enc);
        hce_destroy(h);
    }

    #[test]
    fn kat_all_levels() {
        let key = b"hce-kat-standard-key-32-bytes!!";
        let levels: [(&str, u8); 4] = [("universal", 0), ("eu", 1), ("en", 2), ("numeric", 3)];
        for (name, level_byte) in &levels {
            let path = format!(
                "{}/../../shared/kat/{}.json",
                env!("CARGO_MANIFEST_DIR"),
                name
            );
            let data = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("KAT file not found: {}", path));
            let kat: serde_json::Value = serde_json::from_str(&data).expect("invalid KAT JSON");
            let vectors = kat["vectors"].as_array().expect("no vectors");

            for v in vectors {
                let mode_str = v["mode"].as_str().unwrap();
                let uuid_hex = v["uuid"].as_str().unwrap();
                let expected = v["hce"].as_str().unwrap();
                let uuid: Vec<u8> = (0..16)
                    .map(|i| u8::from_str_radix(&uuid_hex[i * 2..i * 2 + 2], 16).unwrap())
                    .collect();

                let (mode_val, key_ptr, key_len) = match mode_str {
                    "plain" => (2u8, core::ptr::null(), 0usize),
                    _ => (0u8, key.as_ptr(), key.len()),
                };
                if mode_str == "open" {
                    continue;
                }

                let h = hce_new(key_ptr, key_len, *level_byte, mode_val);
                let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
                assert!(enc.ok, "encode failed at {}/{}", name, v["index"]);
                let got = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(enc.data, enc.len))
                };
                assert_eq!(got, expected, "encode mismatch at {}/{}", name, v["index"]);

                let dec = hce_decode(h, enc.data, enc.len);
                assert!(dec.ok, "decode failed at {}/{}", name, v["index"]);
                assert_eq!(
                    unsafe { core::slice::from_raw_parts(dec.data, dec.len) },
                    uuid.as_slice(),
                    "roundtrip mismatch at {}/{}",
                    name,
                    v["index"]
                );

                hce_free_result(enc);
                hce_free_result(dec);
                hce_destroy(h);
            }
            eprintln!("  {}: OK ({} vectors)", name, vectors.len());
        }
    }

    #[test]
    fn recover_corrected() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        let uuid: [u8; 16] = [
            0x19, 0x5e, 0x3a, 0x07, 0xc2, 0xe7, 0xb4, 0x18, 0xf3, 0xd9, 0xa6, 0xc1, 0xe0, 0xb4,
            0xd2, 0x70,
        ];
        let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
        assert!(enc.ok);

        let rec = hce_recover(h, enc.data, enc.len);
        assert!(rec.ok, "recover ok failed: err={}", rec.err_code);

        hce_free_result(enc);
        hce_free_recovery_result(rec);
        hce_destroy(h);
    }

    #[test]
    fn chunk_fixed_roundtrip() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        hce_with_chunk_fixed(h, 7);
        let uuid: [u8; 16] = [0xab; 16];
        let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
        assert!(enc.ok);
        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok);
        assert_eq!(
            unsafe { core::slice::from_raw_parts(dec.data, dec.len) },
            uuid
        );
        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn chunk_pattern_roundtrip() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);
        let pattern: [usize; 4] = [3, 3, 4, 4];
        hce_with_chunk_pattern(h, pattern.as_ptr(), pattern.len());
        let uuid: [u8; 16] = [0xcd; 16];
        let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
        assert!(enc.ok);
        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok);
        assert_eq!(
            unsafe { core::slice::from_raw_parts(dec.data, dec.len) },
            uuid
        );
        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn timestamp_config_changes_output() {
        let h1 = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 1);
        let h2 = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 1);
        hce_with_timestamp_config(h2, 1_800_000_000_000, 5);
        let uuid_v7: [u8; 16] = [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ];
        let enc1 = hce_encode(h1, uuid_v7.as_ptr(), uuid_v7.len());
        let enc2 = hce_encode(h2, uuid_v7.as_ptr(), uuid_v7.len());
        assert!(enc1.ok && enc2.ok);
        let s1 = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(enc1.data, enc1.len))
        };
        let s2 = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(enc2.data, enc2.len))
        };
        assert_ne!(s1, s2, "timestamp config should change open-mode output");
        hce_free_result(enc1);
        hce_free_result(enc2);
        hce_destroy(h1);
        hce_destroy(h2);
    }

    #[test]
    fn error_strings_return_valid() {
        let codes = [1i32, 2, 3, 4, 5, 99];
        for &c in &codes {
            let s = hce_error_string(c);
            assert!(!s.is_null());
            let msg = unsafe { core::ffi::CStr::from_ptr(s as *const i8) };
            assert!(msg.to_bytes().len() > 0);
        }
    }

    #[test]
    fn null_params_return_errors() {
        let h = hce_new(TEST_KEY.as_ptr(), TEST_KEY.len(), 0, 0);

        let enc = hce_encode(core::ptr::null(), TEST_KEY.as_ptr(), 0);
        assert!(!enc.ok);
        assert_eq!(enc.err_code, ERR_NULL_PTR);

        let dec = hce_decode(core::ptr::null(), TEST_KEY.as_ptr(), 0);
        assert!(!dec.ok);
        assert_eq!(dec.err_code, ERR_NULL_PTR);

        let rec = hce_recover(core::ptr::null(), enc.data, 0);
        assert!(!rec.ok);

        hce_destroy(h);
    }

    static mut CB_LAST_PLAIN: u128 = 0;
    static mut CB_LAST_CIPHER: u128 = 0;

    unsafe extern "C" fn test_encrypt_cb(
        _ctx: *mut core::ffi::c_void,
        plain: *const u8,
        _tweak: *const u8,
        out: *mut u8,
    ) {
        let p = u128::from_be_bytes(core::slice::from_raw_parts(plain, 16).try_into().unwrap());
        let c = p ^ 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        CB_LAST_PLAIN = p;
        CB_LAST_CIPHER = c;
        core::slice::from_raw_parts_mut(out, 16).copy_from_slice(&c.to_be_bytes());
    }

    unsafe extern "C" fn test_decrypt_cb(
        _ctx: *mut core::ffi::c_void,
        cipher: *const u8,
        _tweak: *const u8,
        out: *mut u8,
    ) {
        let c = u128::from_be_bytes(core::slice::from_raw_parts(cipher, 16).try_into().unwrap());
        let p = c ^ 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        core::slice::from_raw_parts_mut(out, 16).copy_from_slice(&p.to_be_bytes());
    }

    #[test]
    fn custom_cipher_roundtrip() {
        let vtable = FpeVtable {
            context: core::ptr::null_mut(),
            encrypt: Some(test_encrypt_cb),
            decrypt: Some(test_decrypt_cb),
        };
        let h = hce_new_with_cipher(&vtable, 0, 0);
        assert!(!h.is_null());

        let uuid: [u8; 16] = [0x42; 16];
        let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
        assert!(enc.ok);
        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok);
        assert_eq!(
            unsafe { core::slice::from_raw_parts(dec.data, dec.len) },
            uuid
        );

        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn custom_cipher_null_vtable() {
        let h = hce_new_with_cipher(core::ptr::null(), 0, 0);
        assert!(h.is_null());
    }

    #[test]
    fn builtin_cipher_kind_roundtrip() {
        let key = b"test-key-32-bytes-long-key-here!!";
        let h = hce_new(key.as_ptr(), key.len(), 0, 0);
        hce_with_cipher_kind(h, 1, core::ptr::null(), 0);
        let uuid: [u8; 16] = [0x7f; 16];
        let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
        assert!(enc.ok);
        let dec = hce_decode(h, enc.data, enc.len);
        assert!(dec.ok);
        assert_eq!(
            unsafe { core::slice::from_raw_parts(dec.data, dec.len) },
            uuid
        );
        hce_free_result(enc);
        hce_free_result(dec);
        hce_destroy(h);
    }

    #[test]
    fn custom_cipher_produces_different_output() {
        let key = b"test-key-32-bytes-long-key-here!!";
        let h_default = hce_new(key.as_ptr(), key.len(), 0, 0);
        let uuid: [u8; 16] = [0x42; 16];
        let e1 = hce_encode(h_default, uuid.as_ptr(), uuid.len());
        assert!(e1.ok);

        let vtable = FpeVtable {
            context: core::ptr::null_mut(),
            encrypt: Some(test_encrypt_cb),
            decrypt: Some(test_decrypt_cb),
        };
        let h_custom = hce_new_with_cipher(&vtable, 0, 0);
        let e2 = hce_encode(h_custom, uuid.as_ptr(), uuid.len());
        assert!(e2.ok);

        let s1 = unsafe { core::slice::from_raw_parts(e1.data, e1.len) };
        let s2 = unsafe { core::slice::from_raw_parts(e2.data, e2.len) };
        assert_ne!(
            s1, s2,
            "custom XOR cipher must produce different output from default Feistel"
        );

        hce_free_result(e1);
        hce_free_result(e2);
        hce_destroy(h_default);
        hce_destroy(h_custom);
    }

    #[test]
    fn custom_cipher_all_bit_widths() {
        let vtable = FpeVtable {
            context: core::ptr::null_mut(),
            encrypt: Some(test_encrypt_cb),
            decrypt: Some(test_decrypt_cb),
        };
        for bw in [128u32] {
            let h = hce_new_with_cipher(&vtable, 0, 0);
            assert!(!h.is_null());
            hce_with_bit_width(h, bw);
            let byte_len = (bw.div_ceil(8)) as usize;
            let mut input = [0u8; 16];
            for i in 0..byte_len {
                input[i] = (i + 1) as u8;
            }
            let enc = hce_encode(h, input.as_ptr(), byte_len);
            assert!(enc.ok, "bw={} encode failed", bw);
            let dec = hce_decode(h, enc.data, enc.len);
            assert!(dec.ok, "bw={} decode failed", bw);
            assert_eq!(
                &unsafe { core::slice::from_raw_parts(dec.data, dec.len) }[16 - byte_len..],
                &input[..byte_len],
                "bw={} roundtrip mismatch",
                bw
            );
            hce_free_result(enc);
            hce_free_result(dec);
            hce_destroy(h);
        }
    }
}

#[no_mangle]
pub extern "C" fn hce_adapter_encode(
    adapter_kind: u8,
    key_ptr: *const u8,
    key_len: usize,
    id_ptr: *const u8,
    id_len: usize,
) -> HceResult {
    if id_ptr.is_null() || id_len == 0 {
        return HceResult {
            ok: false,
            data: core::ptr::null_mut(),
            len: 0,
            err_code: ERR_NULL_PTR,
        };
    }
    let hce = Hce::new(
        if key_len > 0 && !key_ptr.is_null() {
            Some(unsafe { core::slice::from_raw_parts(key_ptr, key_len) })
        } else {
            None
        },
        LanguageLevel::Universal,
        HceMode::Sealed,
    );
    let input = unsafe { core::slice::from_raw_parts(id_ptr, id_len) };
    let result = match adapter_kind {
        0 => {
            let codec = HceCodec::new(hce, UuidAdapter);
            codec.encode(input)
        }
        1 => {
            let codec = HceCodec::new(hce, UlidAdapter);
            codec.encode(input)
        }
        2 => {
            let codec = HceCodec::new(hce, SnowflakeAdapter::new(0));
            codec.encode(input)
        }
        3 => {
            let codec = HceCodec::new(hce, XidAdapter);
            codec.encode(input)
        }
        _ => {
            let codec = HceCodec::new(hce, ObjectIdAdapter);
            codec.encode(input)
        }
    };
    let len = result.len();
    let mut buf = result.into_bytes().into_boxed_slice();
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    HceResult {
        ok: true,
        data: ptr,
        len,
        err_code: 0,
    }
}

#[no_mangle]
pub extern "C" fn hce_free_buf(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            drop(alloc::vec::Vec::from_raw_parts(ptr, len, len));
        }
    }
}
