# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qdk.estimator._estimator import LogicalCounts
from qdk.qre._trace import PSSPC, LatticeSurgery
from qdk.qre.application._qsharp import QSharpApplication
from qdk.qre.models.factories._round_based import RoundBasedFactory
from qdk.qre.models.qec._surface_code import SurfaceCode
from qdk.qre.models.qec._yoked import TwoDimensionalYokedSurfaceCode
from qdk.qre.models.qubits._gate_based import GateBased
from qdk.qre4.resource_estimator import ResourceEstimator


def test_new_api():
    # Prepare resource estiator.
    re = ResourceEstimator()
    re.set_qpu(GateBased(gate_time=50, measurement_time=100))
    re.set_compilation_strategy(PSSPC.q() * LatticeSurgery.q(slow_down_factor=[1.0, 2.0]))
    re.add_code(SurfaceCode.q(distance=[7, 11]))
    re.add_factory(RoundBasedFactory.q())
    re.add_code(
        TwoDimensionalYokedSurfaceCode.q(source=SurfaceCode.q(distance=[7, 11]))
    )
    re.options.max_error = 1 / 3

    counts = LogicalCounts(
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

    app = QSharpApplication(counts)

    results = re.estimate(app).estimation_table
    results.add_factory_summary_column()

    assert [(result.qubits, result.runtime) for result in results] == [
        (238707, 23997050000000),
        (240407, 11998525000000),
    ]

    # for result in results:
    #    print("AAAA", result.source)

    print()
    print(results.stats)
