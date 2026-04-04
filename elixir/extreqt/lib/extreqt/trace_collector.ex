defmodule Extreqt.TraceCollector do
  @moduledoc """
  Collects FR traces during test runs.
  """

  use Agent

  def start_link(_opts) do
    Agent.start_link(fn -> %{} end, name: __MODULE__)
  end

  def record(fr_id) do
    test_name = ExUnit.Case.registered_test(__MODULE__)
    
    Agent.update(__MODULE__, fn traces ->
      Map.update(traces, test_name, [fr_id], fn ids -> [fr_id | ids] end)
    end)

    if System.get_env("VERBOSE") do
      IO.puts("[TRACE] #{test_name} -> #{fr_id}")
    end
  end

  def get_traces do
    Agent.get(__MODULE__, & &1)
  end

  def reset do
    Agent.update(__MODULE__, fn _ -> %{} end)
  end

  def validate_fr_id(fr_id) when is_binary(fr_id) do
    Regex.match?(~r/^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$/, fr_id)
  end
end
