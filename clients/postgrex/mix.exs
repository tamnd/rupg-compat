defmodule Scenario.MixProject do
  use Mix.Project

  def project do
    [app: :scenario, version: "0.1.0", elixir: "~> 1.18", deps: [{:postgrex, "0.22.4"}, {:jason, "1.4.5"}]]
  end

  def application, do: [extra_applications: [:logger]]
end
