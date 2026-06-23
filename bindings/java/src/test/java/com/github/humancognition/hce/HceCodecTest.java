package com.github.tmddn3070.hce;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class HceCodecTest {

    static byte[] KEY = "test-key-32-bytes-for-java-test!".getBytes();
    static byte[] UUID = {1, -107, -29, -96, 124, 46, 123, 65, -113, 61, -102, 108, 30, 11, 77, 39};

    @Test void sealedRoundtrip() {
        try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
            var enc = h.encode(UUID);
            assertNotNull(enc);
            assertFalse(enc.isEmpty());
            assertArrayEquals(UUID, h.decode(enc));
        }
    }

    @Test void plainRoundtrip() {
        try (var h = new HceCodec(null, HceCodec.UNIVERSAL, HceCodec.PLAIN)) {
            assertArrayEquals(UUID, h.decode(h.encode(UUID)));
        }
    }

    @Test void openRoundtrip() {
        try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.OPEN)) {
            var enc = h.encode(UUID);
            assertTrue(enc.startsWith("K"));
            assertArrayEquals(UUID, h.decode(enc));
        }
    }

    @Test void allLevels() {
        for (int lv : new int[]{HceCodec.UNIVERSAL, HceCodec.EU, HceCodec.EN, HceCodec.NUMERIC}) {
            try (var h = new HceCodec(KEY, lv, HceCodec.SEALED)) {
                assertArrayEquals(UUID, h.decode(h.encode(UUID)));
            }
        }
    }

    @Test void allBitWidths() {
        for (int bw : new int[]{16, 32, 48, 64, 80, 96, 112, 128}) {
            try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
                h.withBitWidth(bw);
                int blen = (bw + 7) / 8;
                byte[] data = new byte[blen];
                for (int i = 0; i < blen; i++) data[i] = (byte) ((i * 17 + 1) % 256);
                var enc = h.encode(data);
                var dec = h.decode(enc);
                for (int i = 0; i < blen; i++)
                    assertEquals(data[i], dec[16 - blen + i], "bw=" + bw);
            }
        }
    }

    @Test void withCase() {
        try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
            h.withCase(HceCodec.CASE_LOWER);
            var enc = h.encode(UUID);
            for (char c : enc.toCharArray())
                if (c != '-' && c != '.') assertFalse(Character.isUpperCase(c));
            assertArrayEquals(UUID, h.decode(enc));
        }
    }

    @Test void chunkModes() {
        try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
            h.withChunkNone();
            assertFalse(h.encode(UUID).contains("-"));
        }
        try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
            h.withChunkFixed(7);
            assertArrayEquals(UUID, h.decode(h.encode(UUID)));
        }
        try (var h = new HceCodec(KEY, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
            h.withChunkPattern(3, 3, 4, 4);
            assertArrayEquals(UUID, h.decode(h.encode(UUID)));
        }
    }

    @Test void needsKey() {
        assertThrows(IllegalArgumentException.class,
            () -> new HceCodec(null, HceCodec.UNIVERSAL, HceCodec.SEALED));
    }
}
