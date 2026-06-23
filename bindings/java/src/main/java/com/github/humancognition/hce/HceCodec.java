package com.github.tmddn3070.hce;

import com.sun.jna.*;
import java.util.Arrays;
import java.util.List;

public final class HceCodec implements AutoCloseable {

    public static final int UNIVERSAL = 0, EU = 1, EN = 2, NUMERIC = 3;
    public static final int SEALED = 0, OPEN = 1, PLAIN = 2;
    public static final int CASE_LOWER = 0, CASE_UPPER = 1;
    public static final int GRAN_SECOND = 0, GRAN_MINUTE = 1, GRAN_HOUR = 2,
                            GRAN_DAY = 3, GRAN_WEEK = 4, GRAN_MONTH = 5;

    interface Lib extends Library {
        Lib INSTANCE = Native.load("hce_ffi", Lib.class);

        Pointer hce_new(byte[] key, long keyLen, byte level, byte mode);
        void hce_destroy(Pointer h);
        void hce_with_bit_width(Pointer h, int bits);
        void hce_with_domain_modulus(Pointer h, long hi, long lo);
        void hce_with_cipher_kind(Pointer h, byte kind, byte[] key, long keyLen);
        void hce_with_case(Pointer h, byte caseVal);
        void hce_with_check_syllables(Pointer h, long n);
        void hce_with_separator(Pointer h, byte sep);
        void hce_with_chunk_none(Pointer h);
        void hce_with_chunk_fixed(Pointer h, long charSize);
        void hce_with_chunk_pattern(Pointer h, long[] pattern, long count);
        void hce_with_timestamp_config(Pointer h, long epochMs, byte granularity);
        Result.ByValue hce_encode(Pointer h, byte[] data, long len);
        Result.ByValue hce_decode(Pointer h, byte[] input, long len);
        RecoveryResult.ByValue hce_recover(Pointer h, byte[] input, long len);
        void hce_free_result(Result.ByValue r);
        void hce_free_recovery_result(RecoveryResult.ByValue r);
    }

    public static class Result extends Structure {
        public byte ok; public Pointer data; public long len; public int errCode;
        @Override protected List<String> getFieldOrder() { return Arrays.asList("ok","data","len","errCode"); }
        public static class ByValue extends Result implements Structure.ByValue {}
        byte[] toBytes() { if (ok == 0 || data == null || len == 0) return new byte[0]; return data.getByteArray(0, (int) len); }
    }

    public static class RecoveryResult extends Structure {
        public byte ok; public Pointer corrected; public long correctedLen; public long candidateCount; public int errCode;
        @Override protected List<String> getFieldOrder() { return Arrays.asList("ok","corrected","correctedLen","candidateCount","errCode"); }
        public static class ByValue extends RecoveryResult implements Structure.ByValue {}
    }

    private final Pointer inner;

    public HceCodec(byte[] key, int level, int mode) {
        if (mode != PLAIN && (key == null || key.length == 0)) throw new IllegalArgumentException("key required");
        byte[] k = key != null ? key : new byte[0];
        var h = Lib.INSTANCE.hce_new(k, k.length, (byte) level, (byte) mode);
        if (h == null) throw new RuntimeException("hce_new failed");
        this.inner = h;
    }

    public String encode(byte[] data) {
        if (data == null || data.length == 0) throw new IllegalArgumentException("data required");
        var r = Lib.INSTANCE.hce_encode(inner, data, data.length);
        if (r.ok == 0) throw new RuntimeException("encode failed: " + r.errCode);
        var out = r.toBytes(); Lib.INSTANCE.hce_free_result(r); return new String(out);
    }
    public byte[] decode(String input) {
        var in = input.getBytes();
        var r = Lib.INSTANCE.hce_decode(inner, in, in.length);
        if (r.ok == 0) throw new RuntimeException("decode failed: " + r.errCode);
        var out = r.toBytes(); Lib.INSTANCE.hce_free_result(r); return out;
    }
    public byte[] recover(String input) {
        var in = input.getBytes();
        var r = Lib.INSTANCE.hce_recover(inner, in, in.length);
        try {
            if (r.ok == 0) throw new RuntimeException("recover failed: " + r.errCode);
            if (r.correctedLen > 0 && r.corrected != null) {
                byte[] out = new byte[(int) r.correctedLen];
                r.corrected.read(0, out, 0, out.length);
                return out;
            }
            return decode(input);
        } finally { Lib.INSTANCE.hce_free_recovery_result(r); }
    }

    public HceCodec withBitWidth(int bits) { Lib.INSTANCE.hce_with_bit_width(inner, bits); return this; }
    public HceCodec withModulus(long modulus) {
        Lib.INSTANCE.hce_with_domain_modulus(inner, modulus >>> 32, modulus & 0xFFFFFFFFL); return this; }
    public HceCodec withCipherFeistel(byte[] key) {
        byte[] k = key != null ? key : new byte[0];
        Lib.INSTANCE.hce_with_cipher_kind(inner, (byte)0, k, k.length); return this; }
    public HceCodec withCipherShuffle() { Lib.INSTANCE.hce_with_cipher_kind(inner, (byte)1, null, 0); return this; }
    public HceCodec withCase(int caseVal) { Lib.INSTANCE.hce_with_case(inner, (byte) caseVal); return this; }
    public HceCodec withCheckSyllables(int n) { Lib.INSTANCE.hce_with_check_syllables(inner, n); return this; }
    public HceCodec withSeparator(char sep) { Lib.INSTANCE.hce_with_separator(inner, (byte) sep); return this; }
    public HceCodec withChunkNone() { Lib.INSTANCE.hce_with_chunk_none(inner); return this; }
    public HceCodec withChunkFixed(int charSize) { Lib.INSTANCE.hce_with_chunk_fixed(inner, charSize); return this; }
    public HceCodec withChunkPattern(int... pattern) {
        var p = new long[pattern.length]; for (int i = 0; i < pattern.length; i++) p[i] = pattern[i];
        Lib.INSTANCE.hce_with_chunk_pattern(inner, p, p.length); return this; }
    public HceCodec withTimestampConfig(long epochMs, int gran) {
        Lib.INSTANCE.hce_with_timestamp_config(inner, epochMs, (byte) gran); return this; }

    @Override public void close() { if (inner != null) Lib.INSTANCE.hce_destroy(inner); }
}
