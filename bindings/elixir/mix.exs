defmodule Hce.MixProject do
  use Mix.Project

  def project do
    [
      app: :hce,
      version: "0.1.0",
      elixir: "~> 1.16",
      start_permanent: Mix.env() == :prod,
      compilers: [:rustler] ++ Mix.compilers(),
      rustler_crates: [hce_nif: []],
      deps: deps()
    ]
  end

  def application, do: [extra_applications: [:logger]]

  defp deps, do: [{:rustler, "~> 0.35"}]
end
