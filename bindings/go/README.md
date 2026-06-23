# HCE — Go

## Install

```bash
CGO_LDFLAGS="-L$(pwd)/target/release -lhce_ffi" go get github.com/humancognition/hce
```

Build hce-ffi first:

```bash
cargo build -p hce-ffi --release
```

## Usage

```go
package main

import (
    "fmt"
    hce "github.com/humancognition/hce"
)

func main() {
    key := []byte("32-byte-secret-key-here-xxxxxx!!")
    uuid := []byte{1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39}

    c, _ := hce.New(key, hce.Universal, hce.Sealed)
    defer c.Close()

    encoded, _ := c.Encode(uuid)
    fmt.Println(encoded)

    decoded, _ := c.Decode(encoded)
    fmt.Printf("%x\n", decoded)
}
```

## API

```go
func New(key []byte, level Level, mode Mode) (*Codec, error)
func (c *Codec) Close()
func (c *Codec) Encode(data []byte) (string, error)
func (c *Codec) Decode(input string) ([]byte, error)
func (c *Codec) WithBitWidth(bits uint32)
func (c *Codec) WithCase(cas Case)
func (c *Codec) WithCheckSyllables(n int)
func (c *Codec) WithSeparator(sep byte)
func (c *Codec) WithChunkNone()
func (c *Codec) WithChunkFixed(charSize int)
func (c *Codec) WithChunkPattern(pattern []int)
func (c *Codec) WithTimestampConfig(epochMs int64, gran Granularity)
```

## Test

```bash
LD_LIBRARY_PATH=target/release go test -v ./...
```

## License

MIT
