defmodule HceTest do
  use ExUnit.Case
  doctest Hce

  @key "test-key-32-bytes-for-elixir-test!"
  @uuid <<1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39>>

  test "sealed roundtrip" do
    {:ok, h} = Hce.new(@key, :universal, :sealed)
    encoded = Hce.encode(h, @uuid)
    assert is_binary(encoded)
    assert byte_size(encoded) > 0
    {:ok, decoded} = Hce.decode(h, encoded)
    assert decoded == @uuid
    Hce.destroy(h)
  end

  test "plain roundtrip" do
    {:ok, h} = Hce.new(nil, :universal, :plain)
    encoded = Hce.encode(h, @uuid)
    {:ok, decoded} = Hce.decode(h, encoded)
    assert decoded == @uuid
    Hce.destroy(h)
  end

  test "sealed requires key" do
    assert {:error, _} = Hce.new(nil, :universal, :sealed)
  end
end
