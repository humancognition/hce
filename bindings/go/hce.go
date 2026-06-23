package hce

/*
#cgo LDFLAGS: -L${SRCDIR}/../../target/release -lhce_ffi -ldl -lm
#include "../../crates/hce-ffi/hce.h"
#include <stdlib.h>
*/
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

type Level uint8

const (
	Universal Level = C.HCE_LEVEL_UNIVERSAL
	Eu        Level = C.HCE_LEVEL_EU
	En        Level = C.HCE_LEVEL_EN
	Numeric   Level = C.HCE_LEVEL_NUMERIC
)

type Mode uint8

const (
	Sealed Mode = C.HCE_MODE_SEALED
	Open   Mode = C.HCE_MODE_OPEN
	Plain  Mode = C.HCE_MODE_PLAIN
)

type Case uint8

const (
	Lower Case = C.HCE_CASE_LOWER
	Upper Case = C.HCE_CASE_UPPER
)

type Granularity uint8

const (
	GranSecond Granularity = C.HCE_GRANULARITY_SECOND
	GranMinute Granularity = C.HCE_GRANULARITY_MINUTE
	GranHour   Granularity = C.HCE_GRANULARITY_HOUR
	GranDay    Granularity = C.HCE_GRANULARITY_DAY
	GranWeek   Granularity = C.HCE_GRANULARITY_WEEK
	GranMonth  Granularity = C.HCE_GRANULARITY_MONTH
)

var (
	ErrEncode      = errors.New("hce: encode failed")
	ErrDecode      = errors.New("hce: decode failed")
	ErrNullPointer = errors.New("hce: null pointer")
)

type Codec struct {
	inner *C.struct_RawHce
}

func New(key []byte, level Level, mode Mode) (*Codec, error) {
	var keyPtr *C.uint8_t
	var keyLen C.size_t
	if len(key) > 0 {
		keyPtr = (*C.uint8_t)(unsafe.Pointer(&key[0]))
		keyLen = C.size_t(len(key))
	}
	if mode != Plain && len(key) == 0 {
		return nil, fmt.Errorf("hce: key required for %d mode", mode)
	}
	h := C.hce_new(keyPtr, keyLen, C.uint8_t(level), C.uint8_t(mode))
	if h == nil {
		return nil, errors.New("hce: new failed")
	}
	return &Codec{inner: h}, nil
}

func (c *Codec) Close() {
	if c.inner != nil {
		C.hce_destroy(c.inner)
		c.inner = nil
	}
}

func (c *Codec) WithBitWidth(bits uint32) {
	C.hce_with_bit_width(c.inner, C.uint32_t(bits))
}

func (c *Codec) WithCase(cas Case) {
	C.hce_with_case(c.inner, C.uint8_t(cas))
}

func (c *Codec) WithCheckSyllables(n int) {
	C.hce_with_check_syllables(c.inner, C.size_t(n))
}

func (c *Codec) WithSeparator(sep byte) {
	C.hce_with_separator(c.inner, C.uint8_t(sep))
}

func (c *Codec) WithChunkNone() {
	C.hce_with_chunk_none(c.inner)
}

func (c *Codec) WithChunkFixed(charSize int) {
	C.hce_with_chunk_fixed(c.inner, C.size_t(charSize))
}

func (c *Codec) WithChunkPattern(pattern []int) {
	C.hce_with_chunk_pattern(c.inner, (*C.size_t)(unsafe.Pointer(&pattern[0])), C.size_t(len(pattern)))
}

func (c *Codec) WithTimestampConfig(epochMs int64, gran Granularity) {
	C.hce_with_timestamp_config(c.inner, C.int64_t(epochMs), C.uint8_t(gran))
}

func (c *Codec) WithModulus(hi uint64, lo uint64) {
	C.hce_with_domain_modulus(c.inner, C.uint64_t(hi), C.uint64_t(lo))
}

func (c *Codec) WithCipherFeistel(key []byte) {
	var kp *C.uint8_t
	var kl C.size_t
	if len(key) > 0 {
		kp = (*C.uint8_t)(unsafe.Pointer(&key[0]))
		kl = C.size_t(len(key))
	}
	C.hce_with_cipher_kind(c.inner, 0, kp, kl)
}

func (c *Codec) WithCipherShuffle() {
	C.hce_with_cipher_kind(c.inner, 1, nil, 0)
}

func (c *Codec) Recover(input string) ([]byte, error) {
	cs := C.CString(input)
	defer C.free(unsafe.Pointer(cs))
	r := C.hce_recover(c.inner, (*C.uint8_t)(unsafe.Pointer(cs)), C.size_t(len(input)))
	defer C.hce_free_recovery_result(r)
	if !r.ok { return nil, fmt.Errorf("hce: recover error %d", int(r.err_code)) }
	if r.corrected_len > 0 && r.corrected != nil {
		out := make([]byte, int(r.corrected_len))
		copy(out, C.GoBytes(unsafe.Pointer(r.corrected), C.int(r.corrected_len)))
		return out, nil
	}
	return c.Decode(input)
}

func (c *Codec) Encode(data []byte) (string, error) {
	if len(data) == 0 {
		return "", errors.New("hce: empty input")
	}
	r := C.hce_encode(c.inner, (*C.uint8_t)(unsafe.Pointer(&data[0])), C.size_t(len(data)))
	if !r.ok {
		return "", fmt.Errorf("hce: encode error %d", int(r.err_code))
	}
	defer C.hce_free_result(r)
	return C.GoStringN((*C.char)(unsafe.Pointer(r.data)), C.int(r.len)), nil
}

func (c *Codec) Decode(input string) ([]byte, error) {
	cs := C.CString(input)
	defer C.free(unsafe.Pointer(cs))
	r := C.hce_decode(c.inner, (*C.uint8_t)(unsafe.Pointer(cs)), C.size_t(len(input)))
	if !r.ok {
		return nil, fmt.Errorf("hce: decode error %d", int(r.err_code))
	}
	defer C.hce_free_result(r)
	out := make([]byte, int(r.len))
	copy(out, C.GoBytes(unsafe.Pointer(r.data), C.int(r.len)))
	return out, nil
}
