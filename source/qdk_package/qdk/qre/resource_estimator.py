# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from dataclasses import dataclass, field
from types import NoneType
from typing import Any, cast

from ._application import Application
from ._architecture import QPU
from ._estimation import _zero_results_warning_message
from ._isa_enumeration import ISAQuery
from ._qre import ErrorComposition, Trace, _estimate_with_graph, instruction_name
from ._results import EstimationTable, EstimationTableEntry
from ._trace import TraceQuery
from .dollar_cost import DollarCostModel


@dataclass
class ResourceEstimationOptions:
    """Configures constraints applied during resource estimation."""

    max_error: float = 1.0
    error_composition: ErrorComposition = field(
        default_factory=lambda: ErrorComposition.UnionBound
    )


def _logical_counts_from_trace(trace: Trace) -> dict[str, Any]:
    """Extract logical resource counts from an application trace."""
    gates = {
        name: count
        for gate_id, count in trace.gate_counts.items()
        if (name := instruction_name(gate_id)) is not None
    }
    return {
        "compute_qubits": trace.compute_qubits,
        "memory_qubits": trace.memory_qubits,
        "total_qubits": trace.total_qubits,
        "depth": trace.depth,
        "num_gates": trace.num_gates,
        "gates": gates,
    }


@dataclass
class ResourceEstimationResult:
    """Contains estimation results and the source program's logical counts."""

    estimation_table: EstimationTable
    logical_counts: dict[str, Any]
    warning: str | None = None


class CompilationStrategy:
    """Builds a sequence of trace transformations."""

    def __init__(self) -> None:
        """Initialize a strategy with the application's untransformed trace."""
        self.trace_query = TraceQuery(NoneType)

    def add_pass(self, trace_query: TraceQuery) -> None:
        """Append a trace transformation pass to the strategy."""
        self.trace_query *= trace_query


class ResourceEstimator:
    """Configures and runs stateful quantum resource estimation."""

    def __init__(self) -> None:
        """Initialize an estimator with no target or implementation choices."""
        self.qpu: QPU | None = None
        self.compilation_strategies: list[CompilationStrategy] = []
        self.isa_query: ISAQuery | None = None
        self.cost_model: DollarCostModel | None = None
        self.options = ResourceEstimationOptions()

    def set_qpu(self, qpu: QPU) -> None:
        """Set the physical quantum architecture."""
        self.qpu = qpu

    def add_compilation_strategy(
        self, compilation_strategy: CompilationStrategy
    ) -> None:
        """Add a strategy for compiling application traces."""
        self.compilation_strategies.append(compilation_strategy)

    def _add_isa_query(self, isa_query: ISAQuery) -> None:
        """Append an implementation query to the estimator's ISA pipeline."""
        if self.isa_query is None:
            self.isa_query = isa_query
        else:
            self.isa_query *= isa_query

    def add_code(self, code: ISAQuery) -> None:
        """Add a quantum error-correction implementation query."""
        self._add_isa_query(code)

    def add_factory(self, factory: ISAQuery) -> None:
        """Add a resource-state factory implementation query."""
        self._add_isa_query(factory)

    def set_cost_model(self, cost_model: DollarCostModel) -> None:
        """Set the model used to add dollar costs to estimates."""
        self.cost_model = cost_model

    def estimate(
        self,
        application: Application,
        *,
        name: str | None = None,
    ) -> ResourceEstimationResult:
        """Estimate resources for an application using the configured strategies."""
        if self.qpu is None:
            raise ValueError("Must set qpu using set_qpu().")
        if not self.compilation_strategies:
            raise ValueError("Must set at least one compilation strategy.")
        if self.isa_query is None:
            raise ValueError("Must set at least one factory or ECC.")

        input_traces = list(application.enumerate_traces())
        logical_counts = (
            _logical_counts_from_trace(input_traces[0])
            if len(input_traces) == 1
            else {}
        )

        app_ctx = application.context()
        traces = [
            cast(Trace, trace)
            for strategy in self.compilation_strategies
            for trace in strategy.trace_query.enumerate(app_ctx)
        ]

        arch_ctx = self.qpu.context()
        self.isa_query.populate(arch_ctx)
        arch_ctx._provenance.build_pareto_index()
        num_isas = arch_ctx._provenance.total_isa_count()

        collection = _estimate_with_graph(
            traces,
            arch_ctx._provenance,
            self.options.max_error,
            False,
            self.options.error_composition,
        )

        table = EstimationTable()
        table.name = name
        if name is not None:
            table.insert_column(0, "name", lambda _: name)
        table.extend(
            EstimationTableEntry.from_result(result, arch_ctx) for result in collection
        )

        if self.cost_model is not None:
            table.add_cost_column(cost_model=self.cost_model)

        table.stats.num_traces = len(traces)
        table.stats.num_isas = num_isas
        table.stats.total_jobs = collection.total_jobs
        table.stats.successful_estimates = collection.successful_estimates
        table.stats.pareto_results = len(collection)

        result = ResourceEstimationResult(
            estimation_table=table,
            logical_counts=logical_counts,
        )
        if not collection:
            result.warning = _zero_results_warning_message(
                collection,
                num_traces=len(traces),
                num_isas=num_isas,
            )
        return result
