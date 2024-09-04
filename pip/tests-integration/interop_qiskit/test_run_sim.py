# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from concurrent.futures import ThreadPoolExecutor
import pytest
from qsharp import TargetProfile, QSharpError

from interop_qiskit import QISKIT_AVAILABLE, SKIP_REASON

if QISKIT_AVAILABLE:
    from qiskit import QuantumCircuit
    from qiskit_aer import AerSimulator
    from qiskit.qasm3 import loads as from_qasm3
    from qiskit.providers import JobStatus
    from qiskit import ClassicalRegister
    from qsharp.interop import QSharpBackend
    from .test_circuits import (
        core_tests_small,
        generate_repro_information,
    )
else:
    core_tests_small = []


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_run_smoke() -> None:
    circuit = QuantumCircuit(2, 2)
    circuit.x(0)
    circuit.cx(0, 1)
    circuit.measure_all(add_bits=False)
    backend = QSharpBackend()
    res = backend.run(circuit, shots=1).result()
    assert res is not None


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_run_state_prep_smoke() -> None:
    circuit = QuantumCircuit(1)
    circuit.initialize([0.6, 0.8])
    circuit.measure_all()
    backend = QSharpBackend()
    res = backend.run(circuit, shots=1).result()
    assert res is not None


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
@pytest.mark.parametrize("circuit_name", core_tests_small)
def test_random(circuit_name: str, request):
    circuit = request.getfixturevalue(circuit_name)
    if str.endswith(circuit_name.lower(), "base"):
        target_profile = TargetProfile.Base
    else:
        target_profile = TargetProfile.Adaptive_RI
    backend = QSharpBackend(target_profile=target_profile)
    try:
        _ = backend.run(circuit, shots=1).result()
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_get_counts_matches_qiskit_simulator():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()
    backend = QSharpBackend(target_profile=target_profile)

    try:
        qasm3 = backend.qasm3(circuit)
        circuit = from_qasm3(qasm3)

        aersim = AerSimulator()
        job = aersim.run(circuit, shots=5)
        qsjob = backend.run(circuit, shots=5)
        qscounts = qsjob.result().get_counts()
        assert qscounts == job.result().get_counts()
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def create_deterministic_test_circuit():
    cr0 = ClassicalRegister(2, "first")
    cr1 = ClassicalRegister(3, "second")
    circuit = QuantumCircuit(5)
    circuit.add_register(cr0)
    circuit.add_register(cr1)
    circuit.x(0)
    circuit.id(1)
    circuit.id(2)
    circuit.x(3)
    circuit.x(4)
    circuit.measure_all(add_bits=False)
    return circuit


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_execution_works_with_threadpool_set_on_backend():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()
    executor = ThreadPoolExecutor(max_workers=4)
    backend = QSharpBackend(target_profile=target_profile, executor=executor, shots=5)

    try:
        job = backend.run(circuit)
        qscounts = job.result().get_counts()
        assert str(qscounts) == "{'110 01': 5}"
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_execution_works_with_threadpool_set_on_run():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()

    backend = QSharpBackend(target_profile=target_profile, shots=5)
    try:
        executor = ThreadPoolExecutor(max_workers=1)
        job = backend.run(circuit, executor=executor)
        qscounts = job.result().get_counts()
        assert str(qscounts) == "{'110 01': 5}"
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_get_counts_matches_qiskit_simulator_multiple_circuits():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()
    circuit2 = create_deterministic_test_circuit()
    backend = QSharpBackend(target_profile=target_profile)

    try:
        qasm3 = backend.qasm3(circuit)
        circuit = from_qasm3(qasm3)

        aersim = AerSimulator()
        job = aersim.run([circuit, circuit2], shots=5)
        qsjob = backend.run([circuit, circuit2], shots=5)
        qscounts = qsjob.result().get_counts()
        assert qscounts == job.result().get_counts()
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex
