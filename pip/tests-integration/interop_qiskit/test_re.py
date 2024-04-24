# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from concurrent.futures import ThreadPoolExecutor
import pytest

from qsharp.estimator import (
    EstimatorParams,
    QubitParams,
    LogicalCounts,
)


from interop_qiskit import QISKIT_AVAILABLE, SKIP_REASON

if QISKIT_AVAILABLE:
    from qiskit import QuantumCircuit
    from qiskit.circuit.library import RGQFTMultiplier
    from qsharp.interop import ReSimulator


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_qsharp_estimation_with_single_params() -> None:
    params = EstimatorParams()
    params.error_budget = 0.333
    params.qubit_params.name = QubitParams.MAJ_NS_E4
    assert params.as_dict() == {
        "qubitParams": {"name": "qubit_maj_ns_e4"},
        "errorBudget": 0.333,
    }
    circuit = QuantumCircuit(10, 10)
    for index in range(10):
        circuit.t(index)
        circuit.measure(index, index)
    sim = ReSimulator()
    res = sim.run(circuit, params=params).result()

    assert res["status"] == "success"
    assert res["physicalCounts"] is not None
    assert res["jobParams"]["qubitParams"]["name"] == "qubit_maj_ns_e4"
    assert res.logical_counts == LogicalCounts(
        {
            "numQubits": 10,
            "tCount": 10,
            "rotationCount": 0,
            "rotationDepth": 0,
            "cczCount": 0,
            "measurementCount": 10,
        }
    )


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_estimate_qiskit_rgqft_multiplier() -> None:
    bitwidth = 4
    circuit = RGQFTMultiplier(num_state_qubits=bitwidth)
    params = EstimatorParams()
    sim = ReSimulator()
    job = sim.run(circuit, params=params)
    res = job.result()
    assert res["status"] == "success"
    assert res.logical_counts == LogicalCounts(
        {
            "numQubits": 16,
            "tCount": 90,
            "rotationCount": 958,
            "rotationDepth": 658,
            "cczCount": 0,
            "ccixCount": 0,
            "measurementCount": 0,
        }
    )


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_estimate_qiskit_rgqft_multiplier_in_threadpool() -> None:
    bitwidth = 4
    circuit = RGQFTMultiplier(num_state_qubits=bitwidth)
    params = EstimatorParams()
    executor = ThreadPoolExecutor(max_workers=1)
    sim = ReSimulator(executor=executor)
    job = sim.run(circuit, params=params)
    res = job.result()
    assert res["status"] == "success"
    assert res.logical_counts == LogicalCounts(
        {
            "numQubits": 16,
            "tCount": 90,
            "rotationCount": 958,
            "rotationDepth": 658,
            "cczCount": 0,
            "ccixCount": 0,
            "measurementCount": 0,
        }
    )
