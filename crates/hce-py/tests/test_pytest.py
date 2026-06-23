import pytest
import json
import os

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
KAT_DIR = os.path.join(PROJECT_ROOT, "shared", "kat")

KEY = b"hce-kat-standard-key-32-bytes!!"
TEST_KEY = b"py-test-key-32-bytes-long-key!!"
UUID = bytes.fromhex("0195e3a07c2e7b418f3d9a6c1e0b4d27")


def load_kat_vectors(level_name):
    path = os.path.join(KAT_DIR, f"{level_name}.json")
    with open(path) as f:
        data = json.load(f)
    return data["vectors"]


@pytest.mark.parametrize("level_name", ["universal", "eu", "en", "numeric"])
def test_kat_verification(level_name, hce_module):
    vectors = load_kat_vectors(level_name)
    for v in vectors:
        uuid_bytes = bytes.fromhex(v["uuid"])
        mode = v["mode"]
        expected = v["hce"]
        key = None if mode == "plain" else KEY
        h = hce_module.Hce(key=key, level=level_name, mode=mode)
        assert h.encode(uuid_bytes) == expected
        assert h.decode(expected) == uuid_bytes


class TestSealed:
    def test_roundtrip(self, sealed):
        enc = sealed.encode(UUID)
        assert sealed.decode(enc) == UUID

    def test_lower_case(self, sealed_lower):
        enc = sealed_lower.encode(UUID)
        assert not any(c.isupper() for c in enc if c != "-")
        assert sealed_lower.decode(enc) == UUID

    def test_check_syllables(self, sealed_cs3):
        enc = sealed_cs3.encode(UUID)
        assert sealed_cs3.decode(enc) == UUID

    def test_chunk_none(self, sealed_none):
        enc = sealed_none.encode(UUID)
        assert "-" not in enc
        assert sealed_none.decode(enc) == UUID

    def test_chunk_fixed(self, sealed_fixed):
        enc = sealed_fixed.encode(UUID)
        assert sealed_fixed.decode(enc) == UUID

    def test_chunk_pattern(self, sealed_pattern):
        enc = sealed_pattern.encode(UUID)
        assert sealed_pattern.decode(enc) == UUID

    def test_separator_dot(self, sealed_dot):
        enc = sealed_dot.encode(UUID)
        assert "." in enc
        assert sealed_dot.decode(enc) == UUID

    def test_empty_input(self, sealed):
        encoded = sealed.encode(b"")
        decoded = sealed.decode(encoded)
        assert len(decoded) == 16

    def test_max_value(self, sealed):
        data = bytes([0xFF] * 16)
        assert sealed.decode(sealed.encode(data)) == data


class TestPlain:
    def test_roundtrip(self, plain):
        enc = plain.encode(UUID)
        assert plain.decode(enc) == UUID

    def test_no_key_needed(self):
        import hce
        h = hce.Hce(key=None, level="universal", mode="plain")
        enc = h.encode(UUID)
        assert h.decode(enc) == UUID


class TestOpen:
    def test_roundtrip(self, open_hce):
        enc = open_hce.encode(UUID)
        assert open_hce.decode(enc) == UUID

    def test_has_prefix(self, open_hce):
        enc = open_hce.encode(UUID)
        assert enc.startswith("K")

    def test_timestamp_config(self, open_ts):
        enc = open_ts.encode(UUID)
        assert open_ts.decode(enc) == UUID


class TestBitWidths:
    @pytest.mark.parametrize("bw", [16, 17, 24, 32, 48, 63, 64, 65, 80, 96, 112, 128])
    def test_roundtrip(self, hce_module, bw):
        h = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed", bit_width=bw)
        byte_len = (bw + 7) // 8
        data = bytes((i * 17 + 1) % 256 for i in range(byte_len))
        enc = h.encode(data)
        dec = h.decode(enc)
        assert dec[16 - byte_len:] == data


class TestModulus:
    @pytest.mark.parametrize("m", [100, 1000, 10000, 100000, 1_000_000])
    def test_roundtrip(self, hce_module, m):
        h = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed").with_modulus(m)
        enc = h.encode(bytes([42]))
        dec = h.decode(enc)
        assert dec[-1] == 42


class TestCustomCipher:
    @pytest.mark.parametrize("bw", [64, 96, 128])
    def test_shuffle_roundtrip(self, hce_module, bw):
        h = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed", bit_width=bw)
        h = h.with_cipher("shuffle")
        byte_len = (bw + 7) // 8
        data = bytes((i * 17 + 1) % 256 for i in range(byte_len))
        enc = h.encode(data)
        dec = h.decode(enc)
        assert dec[16 - byte_len:] == data

    def test_shuffle_different_output(self, hce_module):
        h1 = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed")
        h2 = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed").with_cipher("shuffle")
        enc1 = h1.encode(UUID)
        enc2 = h2.encode(UUID)
        assert enc1 != enc2
        assert h2.decode(enc2) == UUID

    def test_feistel_with_key(self, hce_module):
        h = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed")
        h = h.with_cipher("feistel", TEST_KEY)
        enc = h.encode(UUID)
        assert h.decode(enc) == UUID

    def test_tweak_changes_output(self, hce_module):
        h1 = hce_module.Hce(key=TEST_KEY, level="universal", mode="sealed").with_cipher("shuffle")
        h2 = hce_module.Hce(key=TEST_KEY, level="eu", mode="sealed").with_cipher("shuffle")
        assert h1.encode(UUID) != h2.encode(UUID)


class TestRecover:
    def test_valid_input(self, sealed):
        enc = sealed.encode(UUID)
        recovered = sealed.recover(enc)
        assert recovered == UUID

    def test_reject(self, sealed):
        with pytest.raises(ValueError):
            sealed.recover("XYZZY-NOTVALID")


class TestErrors:
    def test_empty_key_rejected(self, hce_module):
        with pytest.raises(ValueError):
            hce_module.Hce(key=b"", level="universal", mode="sealed")

    def test_invalid_level(self, hce_module):
        with pytest.raises(ValueError):
            hce_module.Hce(level="invalid")

    def test_invalid_mode(self, hce_module):
        with pytest.raises(ValueError):
            hce_module.Hce(mode="invalid")

    def test_invalid_cipher(self, hce_module):
        with pytest.raises(ValueError):
            hce_module.Hce(key=TEST_KEY).with_cipher("unknown")

    def test_decode_invalid(self, sealed):
        with pytest.raises(ValueError):
            sealed.decode("not-a-valid-hce-string")


class TestAllLevels:
    @pytest.mark.parametrize("level", ["universal", "eu", "en", "numeric"])
    def test_roundtrip(self, level, hce_module):
        h = hce_module.Hce(key=TEST_KEY, level=level, mode="sealed")
        enc = h.encode(UUID)
        assert h.decode(enc) == UUID


class TestAllGranularities:
    @pytest.mark.parametrize("gran", ["second", "minute", "hour", "day", "week", "month"])
    def test_roundtrip(self, hce_module, gran):
        h = hce_module.Hce(key=TEST_KEY, level="universal", mode="open")
        h = h.with_timestamp_config(1_700_000_000_000, gran)
        enc = h.encode(UUID)
        assert h.decode(enc) == UUID
