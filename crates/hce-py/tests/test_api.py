import hce

KEY = b"py-test-key-32-bytes-long-key!!"
UUID = bytes.fromhex("0195e3a07c2e7b418f3d9a6c1e0b4d27")

def test_sealed_roundtrip():
    h = hce.Hce(key=KEY, level="universal", mode="sealed")
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_plain_roundtrip():
    h = hce.Hce(key=None, level="universal", mode="plain")
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_open_roundtrip():
    h = hce.Hce(key=KEY, level="universal", mode="open")
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_bit_width_64():
    h = hce.Hce(key=KEY, level="universal", mode="sealed", bit_width=64)
    data = bytes([0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe])
    encoded = h.encode(data)
    decoded = h.decode(encoded)
    assert decoded[8:] == data

def test_lower_case():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_case("lower")
    encoded = h.encode(UUID)
    assert not any(c.isupper() for c in encoded if c != '-')
    assert h.decode(encoded) == UUID

def test_check_syllables():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_check_syllables(3)
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_chunk_none():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_chunk_none()
    encoded = h.encode(UUID)
    assert '-' not in encoded
    assert h.decode(encoded) == UUID

def test_chunk_fixed():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_chunk_fixed(7)
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_chunk_pattern():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_chunk_pattern([4, 4, 4, 2])
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_separator():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_separator(".")
    encoded = h.encode(UUID)
    assert "." in encoded
    assert h.decode(encoded) == UUID

def test_timestamp_config():
    import datetime
    h = hce.Hce(key=KEY, level="universal", mode="open")
    h = h.with_timestamp_config(1_800_000_000, "month")
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID

def test_recover_valid():
    h = hce.Hce(key=KEY, level="universal", mode="sealed")
    encoded = h.encode(UUID)
    recovered = h.recover(encoded)
    assert recovered == UUID

def test_recover_reject():
    h = hce.Hce(key=KEY, level="universal", mode="sealed")
    try:
        h.recover("XYZZY-BADINPUT-NOTVALID")
        assert False, "should have raised"
    except ValueError:
        pass

def test_decode_invalid_raises():
    h = hce.Hce(key=KEY, level="universal", mode="sealed")
    try:
        h.decode("not-a-valid-hce-string")
        assert False, "should have raised"
    except ValueError:
        pass

def test_all_levels():
    for level in ("universal", "eu", "en", "numeric"):
        h = hce.Hce(key=KEY, level=level, mode="sealed")
        encoded = h.encode(UUID)
        assert h.decode(encoded) == UUID

def test_empty_key_rejected():
    try:
        hce.Hce(key=b"", level="universal", mode="sealed")
        assert False, "should have rejected empty key"
    except ValueError:
        pass

def test_invalid_level_rejected():
    try:
        hce.Hce(level="invalid", mode="sealed")
        assert False, "should have rejected"
    except ValueError:
        pass

def test_invalid_mode_rejected():
    try:
        hce.Hce(mode="invalid")
        assert False, "should have rejected"
    except ValueError:
        pass

def test_custom_cipher_shuffle():
    h1 = hce.Hce(key=KEY, level="universal", mode="sealed")
    h2 = hce.Hce(key=KEY, level="universal", mode="sealed").with_cipher("shuffle")
    e1 = h1.encode(UUID)
    e2 = h2.encode(UUID)
    assert e1 != e2, "shuffle cipher must produce different output from default"
    assert h2.decode(e2) == UUID, "shuffle cipher roundtrip"

def test_custom_cipher_feistel():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_cipher("feistel")
    encoded = h.encode(UUID)
    assert h.decode(encoded) == UUID, "feistel cipher roundtrip"

def test_with_modulus():
    small = bytes([42])
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_modulus(1_000_000)
    encoded = h.encode(small)
    decoded = h.decode(encoded)
    assert decoded[-1] == 42, "modulus roundtrip"

def test_cipher_invalid_rejected():
    try:
        hce.Hce(key=KEY).with_cipher("unknown")
        assert False, "should have rejected"
    except ValueError:
        pass

def test_every_bit_width():
    for bw in [16, 17, 24, 32, 48, 63, 64, 65, 80, 96, 112, 128]:
        h = hce.Hce(key=KEY, level="universal", mode="sealed", bit_width=bw)
        byte_len = (bw + 7) // 8
        data = bytes((i * 17 + 1) % 256 for i in range(byte_len))
        encoded = h.encode(data)
        decoded = h.decode(encoded)
        assert decoded[16 - byte_len:] == data, f"bw={bw}"

def test_custom_cipher_all_bit_widths():
    for bw in [64, 96, 128]:
        h = hce.Hce(key=KEY, level="universal", mode="sealed", bit_width=bw).with_cipher("shuffle")
        byte_len = (bw + 7) // 8
        data = bytes((i * 17 + 1) % 256 for i in range(byte_len))
        encoded = h.encode(data)
        decoded = h.decode(encoded)
        assert decoded[16 - byte_len:] == data, f"shuffle bw={bw}"

def test_max_value():
    h = hce.Hce(key=KEY, level="universal", mode="sealed")
    data = bytes([0xFF] * 16)
    assert h.decode(h.encode(data)) == data

def test_empty_input():
    h = hce.Hce(key=KEY, level="universal", mode="sealed")
    encoded = h.encode(b"")
    decoded = h.decode(encoded)
    assert len(decoded) == 16

def test_tweak_changes_output():
    h1 = hce.Hce(key=KEY, level="universal", mode="sealed").with_cipher("shuffle")
    h2 = hce.Hce(key=KEY, level="eu", mode="sealed").with_cipher("shuffle")
    enc1 = h1.encode(UUID)
    enc2 = h2.encode(UUID)
    assert enc1 != enc2, "different levels must produce different output with same cipher"
    assert h1.decode(enc1) == UUID
    assert h2.decode(enc2) == UUID

def test_all_modulus_values():
    for m in [2, 3, 7, 15, 16, 255, 256, 1000, 100000]:
        val = m - 1 if m <= 256 else 42
        vbytes = val.to_bytes((val.bit_length() + 7) // 8, 'big') or b'\x00'
        h = hce.Hce(key=KEY, level="universal", mode="sealed").with_modulus(m)
        encoded = h.encode(vbytes)
        decoded = h.decode(encoded)
        assert decoded[-1] == (val & 0xFF), f"modulus={m} val={val}"

def test_cli_cipher_feistel():
    h = hce.Hce(key=KEY, level="universal", mode="sealed").with_cipher("feistel", KEY)
    assert h.decode(h.encode(UUID)) == UUID

if __name__ == "__main__":
    tests = [v for k, v in globals().items() if k.startswith("test_")]
    passed = 0
    for t in tests:
        try:
            t()
            passed += 1
        except Exception as e:
            print(f"FAIL: {t.__name__}: {e}")
    print(f"{passed}/{len(tests)} tests passed")
