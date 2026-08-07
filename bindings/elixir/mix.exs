defmodule Hce.MixProject do
  use Mix.Project

  def project do
    [
      app: :hce,
      version: "0.1.2",
      elixir: "~> 1.16",
      description: "Lossless, pronounceable, encrypted codec for any identifier",
      start_permanent: Mix.env() == :prod,
      rustler_crates: [hce_nif: []],
      deps: deps(),
      package: [
        licenses: ["MIT"],
        links: %{"GitHub" => "https://github.com/humancognition/hce"}
      ]
    ]
  end

  def application, do: [extra_applications: [:logger]]

  defp deps, do: [{:rustler, "~> 0.35.0"}]
end
