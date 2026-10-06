# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
 
from dataclasses import dataclass, field
from typing import cast

from qdk.qre.dollar_cost import DollarCostModel

from ..qre._application import Application
from ..qre._architecture import QPU
from ..qre._estimation import _zero_results_warning_message, estimate
from ..qre._isa_enumeration import ISAQuery
from ..qre._qre import ErrorComposition, Trace, _estimate_with_graph
from ..qre._results import EstimationTable, EstimationTableEntry
from ..qre._trace import TraceQuery


@dataclass
class ResourceEstimationOptions:
    max_error: float = 1.0
    error_composition: ErrorComposition = field(
        default_factory=lambda: ErrorComposition.UnionBound
    )


@dataclass
class ResourceEstimationResult:
    estimation_table: EstimationTable
    logical_counts: str
    warning: str | None = None


# New resource estimator API.
class ResourceEstimator:
    qpu: None | QPU = None
    trace_query: None | TraceQuery = None
    isa_query: None | ISAQuery = None
    cost_model: None | DollarCostModel = None
    options: ResourceEstimationOptions = ResourceEstimationOptions()

    def set_qpu(self, qpu: QPU):
        self.qpu = qpu

    def set_compilation_strategy(self, trace_query: TraceQuery):
        self.trace_query = trace_query

    def _add_isa_query(self, isa_query: ISAQuery):
        if self.isa_query is None:
            self.isa_query = isa_query
        else:
            self.isa_query = self.isa_query * isa_query

    def add_code(self, factory: ISAQuery):
        self._add_isa_query(factory)

    def add_factory(self, factory: ISAQuery):
        self._add_isa_query(factory)

    def set_cost_model(self, cost_model: DollarCostModel):
        self.cost_model = cost_model

    def estimate(
        self,
        application: Application,
        *,
        name: str | None = None,
    ) -> ResourceEstimationResult:
        if self.qpu is None:
            raise ValueError("Must set qpu using set_qpu().")
        if self.trace_query is None:
            raise ValueError("Must set compilation strategy.")
        if self.isa_query is None:
            raise ValueError("Must set at least one factory or ECC.")

        # Compute all traces.
        app_ctx = application.context()
        traces = cast(list[Trace], list(self.trace_query.enumerate(app_ctx)))

        arch_ctx = self.qpu.context()  # this would call provided_isa.
        self.isa_query.populate(arch_ctx)
        arch_ctx._provenance.build_pareto_index()
        num_isas = arch_ctx._provenance.total_isa_count()

        collection = _estimate_with_graph(
            traces,
            arch_ctx._provenance,
            self.options.max_error,
            False,  # post_process
            self.options.error_composition,
        )
        num_traces = len(traces)

        # Post-process the results and add them to a results table
        table = EstimationTable()

        table.name = name

        if name is not None:
            table.insert_column(0, "name", lambda entry: name)

        table.extend(
            EstimationTableEntry.from_result(result, arch_ctx) for result in collection
        )

        # If cost model is specified, compute dollar costs.
        if self.cost_model is not None:
            table.add_cost_column(cost_model=self.cost_model)

        # Fill in the stats for this estimation run
        table.stats.num_traces = num_traces
        table.stats.num_isas = num_isas
        table.stats.total_jobs = collection.total_jobs
        table.stats.successful_estimates = collection.successful_estimates
        table.stats.pareto_results = len(collection)

        result = ResourceEstimationResult(
            estimation_table=table,
            logical_counts="LogicalCounts",
        )
        if len(collection) == 0:
            result.warning = _zero_results_warning_message(
                collection,
                num_traces=len(traces),
                num_isas=num_isas,
            )
        return result
