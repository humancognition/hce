# HCE — Java

## Install

Add JNA dependency:

```xml
<dependency>
  <groupId>net.java.dev.jna</groupId>
  <artifactId>jna</artifactId>
  <version>5.16.0</version>
</dependency>
```

Copy `libhce_ffi.so` / `libhce_ffi.dylib` / `hce_ffi.dll` to `java.library.path`.

```bash
cargo build -p hce-ffi --release
cp target/release/libhce_ffi.so /usr/lib/
```

## Usage

```java
import com.github.tmddn3070.hce.HceCodec;

byte[] key = "32-byte-secret-key-here-xxxxxx!!".getBytes();
byte[] uuid = {1, -107, -29, -96, 124, 46, 123, 65, -113, 61, -102, 108, 30, 11, 77, 39};

try (HceCodec h = new HceCodec(key, HceCodec.UNIVERSAL, HceCodec.SEALED)) {
    String encoded = h.encode(uuid);
    byte[] decoded = h.decode(encoded);
}
```

## API

```java
new HceCodec(byte[] key, int level, int mode)

String encode(byte[] data)
byte[] decode(String input)

HceCodec withBitWidth(int bits)
HceCodec withCase(int caseVal)
HceCodec withCheckSyllables(int n)
HceCodec withSeparator(char sep)
HceCodec withChunkNone()
HceCodec withChunkFixed(int charSize)
HceCodec withChunkPattern(int... pattern)
HceCodec withTimestampConfig(long epochMs, int granularity)
```

## Test

```bash
javac -cp jna.jar HceCodec.java
java -cp .:jna.jar -Djava.library.path=target/release HceCodec
```

## License

MIT
