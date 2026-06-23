defmodule Hce do
  @moduledoc "Lossless, pronounceable, encrypted codec."

  @levels %{universal: 0, eu: 1, en: 2, numeric: 3}
  @modes  %{sealed: 0, open: 1, plain: 2}
  @cases  %{lower: 0, upper: 1}
  @grans  %{second: 0, minute: 1, hour: 2, day: 3, week: 4, month: 5}

  defstruct [:ref]

  def new(key, level \\ :universal, mode \\ :sealed) do
    level_byte = Map.fetch!(@levels, level)
    mode_byte  = Map.fetch!(@modes, mode)
    key_bin = if is_nil(key) or key == "", do: <<>>, else: key
    if mode != :plain and byte_size(key_bin) == 0 do
      {:error, "key required for #{mode}"}
    else
      {:ok, %Hce{ref: Hce.NIF.hce_new(key_bin, level_byte, mode_byte)}}
    end
  end

  def encode(%Hce{ref: ref}, data) when is_binary(data), do: Hce.NIF.hce_encode(ref, data)
  def decode(%Hce{ref: ref}, input) when is_binary(input), do: Hce.NIF.hce_decode(ref, input)
  def recover(%Hce{ref: ref}, input) when is_binary(input) do
    case Hce.NIF.hce_recover(ref, input) do
      {:ok, <<>>} -> decode(%Hce{ref: ref}, input)
      {:ok, bytes} -> {:ok, bytes}
      {:error, _} = e -> e
    end
  end

  def with_bit_width(%Hce{ref: ref}, bits), do: (Hce.NIF.hce_with_bit_width(ref, bits); %Hce{ref: ref})
  def with_modulus(%Hce{ref: ref}, m), do: (Hce.NIF.hce_with_domain_modulus(ref, m >>> 32, m &&& 0xFFFFFFFF); %Hce{ref: ref})
  def with_cipher_feistel(%Hce{ref: ref}, key \\ nil) do
    key_bin = if is_nil(key), do: <<>>, else: key
    Hce.NIF.hce_with_cipher_kind(ref, 0, key_bin, byte_size(key_bin)); %Hce{ref: ref}
  end
  def with_cipher_shuffle(%Hce{ref: ref}), do: (Hce.NIF.hce_with_cipher_kind(ref, 1, <<>>, 0); %Hce{ref: ref})
  def with_case(%Hce{ref: ref}, case_val), do: (Hce.NIF.hce_with_case(ref, Map.fetch!(@cases, case_val)); %Hce{ref: ref})
  def with_check_syllables(%Hce{ref: ref}, n), do: (Hce.NIF.hce_with_check_syllables(ref, n); %Hce{ref: ref})
  def with_separator(%Hce{ref: ref}, sep) when is_binary(sep) and byte_size(sep) > 0 do
    Hce.NIF.hce_with_separator(ref, :binary.at(sep, 0)); %Hce{ref: ref}
  end
  def with_chunk_none(%Hce{ref: ref}), do: (Hce.NIF.hce_with_chunk_none(ref); %Hce{ref: ref})
  def with_chunk_fixed(%Hce{ref: ref}, n), do: (Hce.NIF.hce_with_chunk_fixed(ref, n); %Hce{ref: ref})
  def with_chunk_pattern(%Hce{ref: ref}, pattern) when is_list(pattern) do
    Hce.NIF.hce_with_chunk_pattern(ref, pattern); %Hce{ref: ref}
  end
  def with_timestamp_config(%Hce{ref: ref}, epoch_ms, gran), do: (Hce.NIF.hce_with_timestamp_config(ref, epoch_ms, Map.fetch!(@grans, gran)); %Hce{ref: ref})

  def destroy(%Hce{ref: ref}), do: (Hce.NIF.hce_destroy(ref); :ok)

  defimpl String.Chars, do: def(to_string(%Hce{}), do: "Hce")
end
