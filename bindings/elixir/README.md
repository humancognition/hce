# HCE — Elixir

## Install

```elixir
def deps do
  [{:hce, path: "bindings/elixir"}]
end
```

Requires `libhce_ffi` built:

```bash
cargo build -p hce-ffi --release
```

## Usage

```elixir
{:ok, h} = Hce.new("32-byte-key-here-xxxxxx!!", :universal, :sealed)

encoded = Hce.encode(h, <<1, 149, 227, 160, 124, 46, 123, 65>>)
{:ok, decoded} = Hce.decode(h, encoded)

h |> Hce.with_case(:lower) |> Hce.with_bit_width(64)
Hce.destroy(h)
```

## API

```elixir
Hce.new(key, level \\ :universal, mode \\ :sealed)
Hce.encode(codec, data)
Hce.decode(codec, input)
Hce.with_bit_width(codec, bits)
Hce.with_case(codec, :lower | :upper)
Hce.with_separator(codec, sep)
Hce.with_chunk_none(codec)
Hce.with_timestamp_config(codec, epoch_ms, :second..:month)
Hce.destroy(codec)
```

## Test

```bash
mix test
```

## License

MIT
