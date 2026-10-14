# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from types import NoneType

from qdk.estimator import LogicalCounts
from qdk.qre import (
    CompilationStrategy,
    LatticeSurgery,
    PSSPC,
    ResourceEstimator,
)
from qdk.qre.application import QSharpApplication
from qdk.qre.models import (
    GateBased,
    RoundBasedFactory,
    SurfaceCode,
    TwoDimensionalYokedSurfaceCode,
)


def _configured_estimator() -> ResourceEstimator:
    """Create the estimator configuration shared by resource estimator tests."""
    strategy = CompilationStrategy()
    strategy.add_pass(PSSPC.q())
    strategy.add_pass(LatticeSurgery.q(slow_down_factor=[1.0, 2.0]))

    estimator = ResourceEstimator()
    estimator.set_qpu(GateBased(gate_time=50, measurement_time=100))
    estimator.add_compilation_strategy(strategy)
    estimator.add_code(SurfaceCode.q(distance=[7, 11]))
    estimator.add_factory(RoundBasedFactory.q())
    estimator.add_code(
        TwoDimensionalYokedSurfaceCode.q(source=SurfaceCode.q(distance=[7, 11]))
    )
    estimator.options.max_error = 1 / 3
    return estimator


def _logical_counts() -> LogicalCounts:
    """Create representative logical counts with compute and memory qubits."""
    return LogicalCounts(
        {
            "numQubits": 1000,
            "tCount": 1_500_000,
            "rotationCount": 0,
            "rotationDepth": 0,
            "cczCount": 1_000_000_000,
            "ccixCount": 0,
            "measurementCount": 25_000_000,
            "numComputeQubits": 200,
            "readFromMemoryCount": 30_000_000,
            "writeToMemoryCount": 30_000_000,
        }
    )


def test_compilation_strategy_starts_with_no_trace_transforms():
    """Verify that compilation passes follow the source application trace."""
    strategy = CompilationStrategy()

    assert strategy.trace_query.sequence == [(NoneType, {})]

    strategy.add_pass(PSSPC.q())

    assert strategy.trace_query.sequence == [(NoneType, {}), (PSSPC, {})]


def test_resource_estimator():
    """Verify the stateful API produces the expected Pareto frontier."""
    result = _configured_estimator().estimate(QSharpApplication(_logical_counts()))
    estimates = result.estimation_table
    estimates.add_factory_summary_column()

    assert [(estimate.qubits, estimate.runtime) for estimate in estimates] == [
        (238707, 23997050000000),
        (240407, 11998525000000),
    ]


def test_resource_estimator_returns_logical_counts():
    """Verify estimation reports logical counts from the untransformed trace."""
    result = _configured_estimator().estimate(QSharpApplication(_logical_counts()))

    assert result.logical_counts == {
        "compute_qubits": 200,
        "memory_qubits": 800,
        "total_qubits": 1000,
        "depth": 1_086_500_000,
        "num_gates": 1_086_500_000,
        "gates": {
            "CCX": 1_000_000_000,
            "MEAS_Z": 25_000_000,
            "READ_FROM_MEMORY": 30_000_000,
            "T": 1_500_000,
            "WRITE_TO_MEMORY": 30_000_000,
        },
    }
