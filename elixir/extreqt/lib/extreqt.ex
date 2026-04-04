defmodule Extreqt do
  @moduledoc """
  Phenotype Traceability for Elixir/ExUnit.

  Provides a `@trace_to` module attribute and test macros for FR traceability.

  ## Usage

  ```elixir
  defmodule MyTest do
    use ExUnit.Case
    use Extreqt

    @trace_to ["FR-EXAMPLE-001"]
    test "feature works" do
      assert true
    end
  end
  ```
  """

  defmacro __using__(_opts) do
    quote do
      import Extreqt, only: [test_tracing: 2, test_tracing: 3]
    end
  end

  @doc """
  Defines a test that traces to a specific FR.

  ## Example

      test_tracing "FR-EXAMPLE-001", "feature works" do
        assert true
      end
  """
  defmacro test_tracing(fr_id, name, do: block) do
    quote do
      test unquote("[#{unquote(fr_id)}] #{unquote(name)}") do
        Extreqt.TraceCollector.record(unquote(fr_id))
        unquote(block)
      end
    end
  end

  defmacro test_tracing(fr_ids, name, do: block) when is_list(fr_ids) do
    quote do
      fr_ids = unquote(fr_ids)
      fr_str = Enum.join(fr_ids, ", ")

      test unquote("[#{fr_str}] #{unquote(name)}") do
        Enum.each(unquote(fr_ids), &Extreqt.TraceCollector.record/1)
        unquote(block)
      end
    end
  end
end
