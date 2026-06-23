package hce

import (
	"bytes"
	"testing"
)

var testKey = []byte("test-key-32-bytes-for-go--tests!")
var katKey = []byte("hce-kat-standard-key-32-bytes!!")
var testUUID = []byte{1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39}

func TestSealedRoundtrip(t *testing.T) {
	c, _ := New(testKey, Universal, Sealed)
	defer c.Close()
	enc, _ := c.Encode(testUUID)
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("sealed roundtrip failed") }
}

func TestPlainRoundtrip(t *testing.T) {
	c, _ := New(nil, Universal, Plain)
	defer c.Close()
	enc, _ := c.Encode(testUUID)
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("plain roundtrip failed") }
}

func TestOpenRoundtrip(t *testing.T) {
	c, _ := New(testKey, Universal, Open)
	defer c.Close()
	enc, _ := c.Encode(testUUID)
	if !bytes.HasPrefix([]byte(enc), []byte("K")) { t.Fatal("open mode should have K prefix") }
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("open roundtrip failed") }
}

func TestAllLevels(t *testing.T) {
	levels := []Level{Universal, Eu, En, Numeric}
	for _, lv := range levels {
		c, _ := New(testKey, lv, Sealed)
		enc, _ := c.Encode(testUUID)
		dec, _ := c.Decode(enc)
		if !bytes.Equal(dec, testUUID) { t.Fatalf("level %d roundtrip failed", lv) }
		c.Close()
	}
}

func TestBitWidths(t *testing.T) {
	for _, bw := range []uint32{16, 32, 48, 64, 80, 96, 112, 128} {
		c, _ := New(testKey, Universal, Sealed)
		c.WithBitWidth(bw)
		blen := int((bw + 7) / 8)
		data := make([]byte, blen)
		for i := 0; i < blen; i++ { data[i] = byte((i*17 + 1) % 256) }
		enc, err := c.Encode(data)
		if err != nil { t.Fatalf("bw=%d encode: %v", bw, err) }
		dec, err := c.Decode(enc)
		if err != nil { t.Fatalf("bw=%d decode: %v", bw, err) }
		if !bytes.Equal(dec[16-blen:], data) { t.Fatalf("bw=%d mismatch", bw) }
		c.Close()
	}
}

func TestWithCase(t *testing.T) {
	for _, cas := range []Case{Lower, Upper} {
		c, _ := New(testKey, Universal, Sealed)
		c.WithCase(cas)
		enc, _ := c.Encode(testUUID)
		dec, _ := c.Decode(enc)
		if !bytes.Equal(dec, testUUID) { t.Fatal("case roundtrip failed") }
		if cas == Lower {
			for _, ch := range enc {
				if ch >= 'A' && ch <= 'Z' { t.Fatal("lowercase output has uppercase") }
			}
		}
		c.Close()
	}
}

func TestCheckSyllables(t *testing.T) {
	for _, n := range []int{1, 2, 4, 8} {
		c, _ := New(testKey, Universal, Sealed)
		c.WithCheckSyllables(n)
		enc, _ := c.Encode(testUUID)
		dec, _ := c.Decode(enc)
		if !bytes.Equal(dec, testUUID) { t.Fatalf("check=%d failed", n) }
		c.Close()
	}
}

func TestChunkModes(t *testing.T) {
	c, _ := New(testKey, Universal, Sealed)
	c.WithChunkNone()
	enc, _ := c.Encode(testUUID)
	if bytes.Contains([]byte(enc), []byte{'-'}) { t.Fatal("chunk_none has hyphens") }
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("chunk_none roundtrip") }
	c.Close()

	c, _ = New(testKey, Universal, Sealed)
	c.WithChunkFixed(7)
	enc, _ = c.Encode(testUUID)
	dec, _ = c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("chunk_fixed roundtrip") }
	c.Close()

	c, _ = New(testKey, Universal, Sealed)
	c.WithChunkPattern([]int{3, 3, 4, 4})
	enc, _ = c.Encode(testUUID)
	dec, _ = c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("chunk_pattern roundtrip") }
	c.Close()
}

func TestSeparator(t *testing.T) {
	c, _ := New(testKey, Universal, Sealed)
	c.WithSeparator('.')
	enc, _ := c.Encode(testUUID)
	if !bytes.Contains([]byte(enc), []byte{'.'}) { t.Fatal("separator dot not in output") }
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("separator roundtrip") }
	c.Close()
}

func TestTimestampConfig(t *testing.T) {
	c, _ := New(testKey, Universal, Open)
	c.WithTimestampConfig(1800000000000, GranMonth)
	enc, _ := c.Encode(testUUID)
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, testUUID) { t.Fatal("timestamp roundtrip") }
	c.Close()
}

func TestSealedNeedsKey(t *testing.T) {
	if _, err := New(nil, Universal, Sealed); err == nil { t.Fatal("expected error") }
}

func TestEmptyInput(t *testing.T) {
	c, _ := New(testKey, Universal, Sealed)
	defer c.Close()
	if _, err := c.Encode([]byte{}); err == nil { t.Fatal("expected error for empty input") }
}

func TestMaxValue(t *testing.T) {
	c, _ := New(testKey, Universal, Sealed)
	defer c.Close()
	data := make([]byte, 16)
	for i := range data { data[i] = 0xFF }
	enc, _ := c.Encode(data)
	dec, _ := c.Decode(enc)
	if !bytes.Equal(dec, data) { t.Fatal("max value roundtrip") }
}
