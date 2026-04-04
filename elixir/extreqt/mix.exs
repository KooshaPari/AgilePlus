defmodule Extreqt.MixProject do
  use Mix.Project

  def project do
    [
      app: :extreqt,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "Feature Requirement (FR) traceability for Elixir/ExUnit tests",
      package: package()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:ex_doc, "~> 0.30", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["Apache-2.0"],
      links: %{"GitHub" => "https://github.com/phenotype/AgilePlus"}
    ]
  end
end
