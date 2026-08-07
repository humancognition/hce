defmodule HceTest do
  use ExUnit.Case

  @key "test-key-32-bytes-for-elixir-test!"
  @v7 <<0x01, 0x95, 0xE3, 0xA0, 0x7C, 0x2E, 0x7B, 0x41, 0x8F, 0x3D, 0x9A, 0x6C, 0x1E, 0x0B, 0x4D, 0x27>>
  @v4 <<0xF8, 0x1D, 0x4F, 0xAE, 0x7D, 0xEC, 0x4D, 0x10, 0xA7, 0x65, 0x00, 0xA0, 0xC9, 0x1E, 0x6B, 0xF6>>

  test "sealed roundtrip v7" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    enc = Hce.encode(h, @v7)
    assert is_binary(enc) and byte_size(enc) > 10
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v7
    Hce.destroy(h)
  end

  test "sealed roundtrip v4" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    enc = Hce.encode(h, @v4)
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v4
    Hce.destroy(h)
  end

  test "plain roundtrip" do
    {:ok, h} = Hce.new(nil, :universal, :plain)
    enc = Hce.encode(h, @v7)
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v7
    Hce.destroy(h)
  end

  test "open has prefix" do
    {:ok, h} = Hce.new(@key, :universal, :open)
    enc = Hce.encode(h, @v7)
    assert String.starts_with?(enc, "B")
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v7
    Hce.destroy(h)
  end

  test "v4 has no open prefix" do
    {:ok, h} = Hce.new(@key, :universal, :open)
    enc = Hce.encode(h, @v4)
    refute String.starts_with?(enc, "B")
    Hce.destroy(h)
  end

  test "sealed requires key" do
    assert {:error, _} = Hce.new(nil, :universal, :sealed)
  end

  test "empty key rejected for sealed" do
    assert {:error, _} = Hce.new("", :universal, :sealed)
  end

  test "invalid level rejected" do
    assert_raise KeyError, fn -> Hce.new(@key, :invalid, :sealed) end
  end

  test "invalid mode rejected" do
    assert_raise KeyError, fn -> Hce.new(@key, :universal, :invalid) end
  end

  test "all levels produce different output" do
    encs = for level <- [:universal, :eu, :en, :numeric, :ko] do
      {:ok, h} = Hce.new(@key, level, :sealed)
      enc = Hce.encode(h, @v7)
      {:ok, dec} = Hce.decode(h, enc)
      assert dec == @v7, "level #{level} roundtrip"
      Hce.destroy(h)
      enc
    end
    assert MapSet.new(encs) |> MapSet.size() == 5
  end

  test "every bit width roundtrip" do
    for bw <- [16, 17, 24, 31, 32, 33, 48, 63, 64, 65, 80, 96, 112, 127, 128] do
      {:ok, h} = Hce.new(@key, :universal, :sealed)
      h = Hce.with_bit_width(h, bw)
      byte_len = div(bw + 7, 8)
      data = for i <- 0..(byte_len - 1), into: <<>>, do: <<rem(i * 17 + 1, 256)>>
      enc = Hce.encode(h, data)
      {:ok, dec} = Hce.decode(h, enc)
      assert binary_part(dec, 16 - byte_len, byte_len) == data, "bw=#{bw}"
      Hce.destroy(h)
    end
  end

  test "lower case has no uppercase" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    h = Hce.with_case(h, :lower)
    enc = Hce.encode(h, @v7)
    refute String.match?(enc, ~r/[A-Z]/)
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v7
    Hce.destroy(h)
  end

  test "check syllables 1-8" do
    for n <- 1..8 do
      {:ok, h} = Hce.new(@key, :universal, :sealed)
      h = Hce.with_check_syllables(h, n)
      enc = Hce.encode(h, @v7)
      {:ok, dec} = Hce.decode(h, enc)
      assert dec == @v7, "check_syllables=#{n}"
      Hce.destroy(h)
    end
  end

  test "chunk none has no separator" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    h = Hce.with_chunk_none(h)
    enc = Hce.encode(h, @v7)
    refute String.contains?(enc, "-")
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v7
    Hce.destroy(h)
  end

  test "chunk fixed roundtrip" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    h = Hce.with_chunk_fixed(h, 7)
    {:ok, dec} = Hce.decode(h, Hce.encode(h, @v7))
    assert dec == @v7
    Hce.destroy(h)
  end

  test "chunk pattern roundtrip" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    h = Hce.with_chunk_pattern(h, [3, 3, 4, 4])
    {:ok, dec} = Hce.decode(h, Hce.encode(h, @v7))
    assert dec == @v7
    Hce.destroy(h)
  end

  test "custom separator dot" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    h = Hce.with_separator(h, ".")
    enc = Hce.encode(h, @v7)
    assert String.contains?(enc, ".")
    {:ok, dec} = Hce.decode(h, enc)
    assert dec == @v7
    Hce.destroy(h)
  end

  test "shuffle cipher different output" do
    {:ok, h1} = Hce.new(@key, :universal, :sealed)
    {:ok, h2} = Hce.new(@key, :universal, :sealed)
    h2 = Hce.with_cipher_shuffle(h2)
    e1 = Hce.encode(h1, @v7)
    e2 = Hce.encode(h2, @v7)
    assert e1 != e2
    {:ok, dec} = Hce.decode(h2, e2)
    assert dec == @v7
    Hce.destroy(h1)
    Hce.destroy(h2)
  end

  test "feistel cipher with key" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    h = Hce.with_cipher_feistel(h, @key)
    {:ok, dec} = Hce.decode(h, Hce.encode(h, @v7))
    assert dec == @v7
    Hce.destroy(h)
  end

  test "recover valid" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    enc = Hce.encode(h, @v7)
    {:ok, rec} = Hce.recover(h, enc)
    assert rec == @v7
    Hce.destroy(h)
  end

  test "recover invalid rejects" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    assert {:error, _} = Hce.recover(h, "XYZZY-BADINPUT")
    Hce.destroy(h)
  end

  test "decode invalid raises" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    assert {:error, _} = Hce.decode(h, "not-valid-hce")
    Hce.destroy(h)
  end

  test "max value roundtrip" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    data = :binary.copy(<<0xFF>>, 16)
    {:ok, dec} = Hce.decode(h, Hce.encode(h, data))
    assert dec == data
    Hce.destroy(h)
  end

  test "zero value roundtrip" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    data = :binary.copy(<<0>>, 16)
    {:ok, dec} = Hce.decode(h, Hce.encode(h, data))
    assert dec == data
    Hce.destroy(h)
  end

  test "modulus roundtrip" do
    for m <- [2, 3, 7, 15, 256, 1000, 100_000] do
      {:ok, h} = Hce.new(@key, :universal, :sealed)
      h = Hce.with_modulus(h, m)
      small = <<42>>
      enc = Hce.encode(h, small)
      {:ok, dec} = Hce.decode(h, enc)
      assert byte_size(dec) == 16
      Hce.destroy(h)
    end
  end

  test "timestamp month granularity" do
    for gran <- [:second, :minute, :hour, :day, :week, :month] do
      {:ok, h} = Hce.new(@key, :universal, :open)
      h = Hce.with_timestamp_config(h, 1_800_000_000_000, gran)
      enc = Hce.encode(h, @v7)
      {:ok, dec} = Hce.decode(h, enc)
      assert dec == @v7, "gran=#{gran}"
      Hce.destroy(h)
    end
  end

  test "decode cross-level fails" do
    {:ok, h_univ} = Hce.new(@key, :universal, :sealed)
    {:ok, h_ko} = Hce.new(@key, :ko, :sealed)
    enc = Hce.encode(h_univ, @v7)
    assert {:error, _} = Hce.decode(h_ko, enc)
    Hce.destroy(h_univ)
    Hce.destroy(h_ko)
  end
end
