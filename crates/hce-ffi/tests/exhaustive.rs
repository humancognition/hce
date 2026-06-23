#[path = "../src/lib.rs"]
mod ffi;

use core::slice;
use ffi::*;

static KEY: &[u8] = b"exhaustive-test-key--32-bytes!!";

fn roundtrip_ffi(h: *mut RawHce, data: &[u8], byte_len: usize) {
    let enc = hce_encode(h, data.as_ptr(), data.len());
    assert!(enc.ok, "encode failed: err={}", enc.err_code);
    let dec = hce_decode(h, enc.data, enc.len);
    assert!(dec.ok, "decode failed: err={}", dec.err_code);
    unsafe {
        assert_eq!(
            &slice::from_raw_parts(dec.data, dec.len)[16 - byte_len..],
            &data[..byte_len.min(data.len())]
        );
    }
    hce_free_result(enc);
    hce_free_result(dec);
}

#[test]
fn ffi_combinatorial_levels_modes() {
    let levels: [u8; 4] = [0, 1, 2, 3];
    let mode_info: [(u8, bool); 3] = [(0, false), (1, false), (2, true)];
    let mut seed: u64 = 0xB00B;
    for &lb in &levels {
        for &(mb, is_plain) in &mode_info {
            let (kp, kl) = if is_plain {
                (core::ptr::null(), 0usize)
            } else {
                (KEY.as_ptr(), KEY.len())
            };
            let h = hce_new(kp, kl, lb, mb);
            assert!(!h.is_null());
            for _ in 0..3 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let val = seed as u128;
                roundtrip_ffi(h, &val.to_be_bytes(), 16);
            }
            hce_destroy(h);
        }
    }
}

#[test]
fn ffi_combinatorial_bit_widths() {
    let mut seed: u64 = 0xF001;
    for bw in [16u32, 32, 64, 96, 128] {
        let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
        hce_with_bit_width(h, bw);
        let blen = (bw.div_ceil(8)) as usize;
        for _ in 0..3 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = seed as u128;
            let data = &val.to_be_bytes()[16 - blen..];
            roundtrip_ffi(h, data, data.len());
        }
        hce_destroy(h);
    }
}

#[test]
fn ffi_combinatorial_ciphers() {
    for kind in [0u8, 1u8] {
        let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
        hce_with_cipher_kind(h, kind, KEY.as_ptr(), KEY.len());
        let uuid: [u8; 16] = [0x42; 16];
        roundtrip_ffi(h, &uuid, 16);
        hce_destroy(h);
    }
}

#[test]
fn ffi_combinatorial_chunks() {
    let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
    hce_with_chunk_none(h);
    let uuid: [u8; 16] = [0x7f; 16];
    let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
    assert!(enc.ok);
    let s = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(enc.data, enc.len)) };
    assert!(!s.contains('-'));
    hce_free_result(enc);
    hce_destroy(h);

    let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
    hce_with_chunk_fixed(h, 7);
    roundtrip_ffi(h, &[0xab; 16], 16);
    hce_destroy(h);

    let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
    let pat: [usize; 4] = [3, 3, 4, 4];
    hce_with_chunk_pattern(h, pat.as_ptr(), pat.len());
    roundtrip_ffi(h, &[0xcd; 16], 16);
    hce_destroy(h);

    let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
    hce_with_separator(h, b'.');
    let enc = hce_encode(h, uuid.as_ptr(), uuid.len());
    assert!(enc.ok);
    let s = unsafe { core::str::from_utf8_unchecked(slice::from_raw_parts(enc.data, enc.len)) };
    assert!(s.contains('.'));
    hce_free_result(enc);
    hce_destroy(h);
}

#[test]
fn ffi_combinatorial_timestamp() {
    for gran in [0u8, 1, 2, 3, 4, 5] {
        let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 1);
        hce_with_timestamp_config(h, 1_800_000_000_000, gran);
        let uuid: [u8; 16] = [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ];
        roundtrip_ffi(h, &uuid, 16);
        hce_destroy(h);
    }
}

#[test]
fn ffi_combinatorial_errors() {
    assert!(hce_new_with_cipher(core::ptr::null(), 0, 0).is_null());
    let h = hce_new(KEY.as_ptr(), KEY.len(), 0, 0);
    let enc = hce_encode(core::ptr::null(), KEY.as_ptr(), 0);
    assert!(!enc.ok && enc.err_code == 5);
    let dec = hce_decode(core::ptr::null(), KEY.as_ptr(), 0);
    assert!(!dec.ok && dec.err_code == 5);
    for c in [1i32, 2, 3, 4, 5, 99] {
        let s = hce_error_string(c);
        assert!(!s.is_null());
    }
    hce_destroy(h);
    assert!(hce_clone(core::ptr::null()).is_null());
}

#[test]
fn ffi_custom_cipher_roundtrip() {
    unsafe extern "C" fn xor_enc(
        _: *mut core::ffi::c_void,
        plain: *const u8,
        _: *const u8,
        out: *mut u8,
    ) {
        let p = u128::from_be_bytes(slice::from_raw_parts(plain, 16).try_into().unwrap());
        let c = p ^ 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        slice::from_raw_parts_mut(out, 16).copy_from_slice(&c.to_be_bytes());
    }
    unsafe extern "C" fn xor_dec(
        _: *mut core::ffi::c_void,
        cipher: *const u8,
        _: *const u8,
        out: *mut u8,
    ) {
        let c = u128::from_be_bytes(slice::from_raw_parts(cipher, 16).try_into().unwrap());
        let p = c ^ 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        slice::from_raw_parts_mut(out, 16).copy_from_slice(&p.to_be_bytes());
    }
    let vt = FpeVtable {
        context: core::ptr::null_mut(),
        encrypt: Some(xor_enc),
        decrypt: Some(xor_dec),
    };
    let h = hce_new_with_cipher(&vt, 0, 0);
    assert!(!h.is_null());
    let uuid: [u8; 16] = [0x42; 16];
    roundtrip_ffi(h, &uuid, 16);
    hce_destroy(h);
}
