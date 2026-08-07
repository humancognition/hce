defmodule Hce.NIF do
  @on_load :load_nif

  def load_nif do
    path = :filename.join(:code.priv_dir(:hce), ~c"native/libhce_nif")
    :erlang.load_nif(path, 0)
  end

  # These are implemented in Rust — the signatures here are for documentation
  def hce_new(_key, _level, _mode), do: :erlang.nif_error(:nif_not_loaded)
  def hce_encode(_ref, _data), do: :erlang.nif_error(:nif_not_loaded)
  def hce_decode(_ref, _input), do: :erlang.nif_error(:nif_not_loaded)
  def hce_recover(_ref, _input), do: :erlang.nif_error(:nif_not_loaded)
  def hce_destroy(_ref), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_bit_width(_ref, _bits), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_domain_modulus(_ref, _hi, _lo), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_cipher_kind(_ref, _kind, _key, _key_len), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_case(_ref, _case_val), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_check_syllables(_ref, _n), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_separator(_ref, _sep), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_chunk_none(_ref), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_chunk_fixed(_ref, _n), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_chunk_pattern(_ref, _pattern), do: :erlang.nif_error(:nif_not_loaded)
  def hce_with_timestamp_config(_ref, _epoch_ms, _gran), do: :erlang.nif_error(:nif_not_loaded)
end
