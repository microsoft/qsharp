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
    from qsharp.interop import QSharpSimulator
    from .test_circuits import (
        core_tests_small,
        generate_repro_information,
        get_parameterized_circuit,
    )
else:
    core_tests_small = []


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_run_smoke() -> None:
    circuit = QuantumCircuit(2, 2)
    circuit.x(0)
    circuit.cx(0, 1)
    circuit.measure_all(add_bits=False)
    backend = QSharpSimulator()
    res = backend.run(circuit, shots=1).result()
    assert res is not None


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_run_state_prep_smoke() -> None:
    circuit = QuantumCircuit(1)
    circuit.initialize([0.6, 0.8])
    circuit.measure_all()
    backend = QSharpSimulator()
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

    try:
        backend = QSharpSimulator()
        _ = backend.run(circuit, target_profile=target_profile, shots=1).result()
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_get_counts_matches_qiskit_simulator():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()

    try:
        backend = QSharpSimulator(target_profile=target_profile)
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
        additional_info = generate_repro_information(circuit, target_profile)
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

    try:
        executor = ThreadPoolExecutor(max_workers=1)
        backend = QSharpSimulator(
            target_profile=target_profile, executor=executor, shots=5
        )

        job = backend.run(circuit)
        qscounts = job.result().get_counts()
        assert str(qscounts) == "{'110 01': 5}"
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_execution_works_with_threadpool_set_on_run():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()

    try:
        backend = QSharpSimulator(target_profile=target_profile, shots=5)

        executor = ThreadPoolExecutor(max_workers=1)
        job = backend.run(circuit, executor=executor)
        qscounts = job.result().get_counts()
        assert str(qscounts) == "{'110 01': 5}"
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_simple_parameter_can_be_passed_for_execution():
    target_profile = TargetProfile.Base
    circuit = get_parameterized_circuit(5)

    backend = QSharpSimulator(target_profile=target_profile, shots=5)

    job = backend.run(circuit, params="0.5")
    job.wait_for_final_state()
    if job.status() != JobStatus.DONE:
        raise RuntimeError("Job Failed") from job.error()


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_error_message_when_entry_expr_is_missing():
    target_profile = TargetProfile.Base
    circuit = get_parameterized_circuit(5)

    backend = QSharpSimulator(target_profile=target_profile, shots=5)
    with pytest.raises(QSharpError) as ex:
        job = backend.run(circuit)
        job.result()

    strex = str(ex.value)
    assert "params are required when the circuit has unbound input parameters" in strex
    assert "help: Parameters: θ: Double" in strex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_error_message_when_entry_expr_params_are_incorrect():
    target_profile = TargetProfile.Base
    circuit = get_parameterized_circuit(5)
    circuit.name = "parameterized_circuit"

    backend = QSharpSimulator(target_profile=target_profile, shots=5)
    with pytest.raises(QSharpError) as ex:
        job = backend.run(circuit, params="5")
        job.result()

    strex = str(ex.value)
    assert "failed to compile entry point" in strex
    assert "expected Double, found Int" in strex
    assert "check that the parameter types match the supplied parameters" in strex
    assert "help: Parameters: θ: Double" in strex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_error_message_when_entry_expr_is_not_found():
    target_profile = TargetProfile.Base
    circuit = get_parameterized_circuit(5)
    circuit.name = "parameterized_circuit"

    backend = QSharpSimulator(target_profile=target_profile, shots=5)
    with pytest.raises(QSharpError) as ex:
        job = backend.run(circuit, params="DoesNotExist()")
        job.result()

    strex = str(ex.value)
    assert "Qsc.Resolve.NotFound" in strex
    assert "failed to compile entry point" in strex
    assert "check that the parameter types match the supplied parameters" in strex
    assert "help: Parameters: θ: Double" in strex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_get_counts_matches_qiskit_simulator_multiple_circuits():
    target_profile = TargetProfile.Base
    circuit = create_deterministic_test_circuit()
    circuit2 = create_deterministic_test_circuit()

    try:
        backend = QSharpSimulator(target_profile=target_profile)
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
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex
