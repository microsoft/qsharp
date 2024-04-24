# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from typing import Optional

import pytest
from qsharp import TargetProfile, QSharpError

from . import QISKIT_AVAILABLE, SKIP_REASON, ignore_on_failure


if QISKIT_AVAILABLE:
    from .test_circuits import (
        core_tests,
        generate_repro_information,
    )
    from qsharp.interop.qiskit import QasmError, QirTarget
    from qiskit.circuit import QuantumCircuit, Parameter, Gate
    from qiskit.circuit.quantumcircuit import QubitSpecifier
    from qsharp.interop.qiskit.backends import QSharpBackend
else:
    core_tests = []


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
@pytest.mark.parametrize("circuit_name", core_tests)
def test_random(circuit_name: str, request):
    circuit = request.getfixturevalue(circuit_name)
    if str.endswith(circuit_name.lower(), "base"):
        target_profile = TargetProfile.Base
    else:
        target_profile = TargetProfile.Adaptive_RI

    backend = QSharpBackend(target_profile=target_profile)
    try:
        qir = backend.qir(circuit)
        assert qir is not None
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_with_unbound_param_raises():
    theta = Parameter("theta")

    circuit = QuantumCircuit(1)
    circuit.name = "test"
    circuit.rx(theta, 0)
    circuit.measure_all()

    target_profile = TargetProfile.Base
    backend = QSharpBackend(target_profile=target_profile)
    try:
        with pytest.raises(QSharpError) as ex:
            _ = backend.qir(circuit)
        message = str(ex.value)
        assert "Circuit has unbound input parameters" in message
        assert "help: Parameters: theta: Double" in message
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_with_bound_param_with_wrong_type_raises():
    theta = Parameter("theta")

    circuit = QuantumCircuit(1)
    circuit.name = "test"
    circuit.rx(theta, 0)
    circuit.measure_all()

    target_profile = TargetProfile.Base
    backend = QSharpBackend(target_profile=target_profile)
    try:
        with pytest.raises(QSharpError) as ex:
            _ = backend.qir(circuit, params="true")
        message = str(ex)
        assert "Circuit has unbound input parameters" in message
        assert "help: Parameters: theta: Double" in message
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_with_bound_param():
    theta = Parameter("theta")

    circuit = QuantumCircuit(1)
    circuit.name = "test"
    circuit.rx(theta, 0)
    circuit.measure_all()
    circuit.assign_parameters({"theta": 0.5}, inplace=True)

    target_profile = TargetProfile.Base
    backend = QSharpBackend(target_profile=target_profile)
    try:
        qir = backend.qir(circuit)
        assert qir is not None
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_without_measuring_into_all_registers_raises():
    circuit = QuantumCircuit(2, 2)
    circuit.rx(0.5, 0)

    target_profile = TargetProfile.Base
    backend = QSharpBackend(target_profile=target_profile)
    try:
        with pytest.raises(QSharpError) as ex:
            _ = backend.qir(circuit)
        assert "ensure all output registers have been measured into." in str(ex)
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_without_registers_raises():
    circuit = QuantumCircuit(2)
    circuit.rx(0.5, 0)

    target_profile = TargetProfile.Base
    backend = QSharpBackend(target_profile=target_profile)
    try:
        with pytest.raises(QasmError) as ex:
            _ = backend.qir(circuit)
        assert "Qiskit circuits must have output registers." in str(ex)
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, backend)
        raise RuntimeError(additional_info) from ex


def get_resource_path(file_name: Optional[str] = None) -> str:
    current_directory = os.path.dirname(os.path.abspath(__file__))
    if file_name is None:
        return os.path.join(current_directory, "resources")
    return os.path.join(current_directory, "resources", file_name)


def read_resource_file(file_name: str) -> str:
    resource_path = get_resource_path(file_name)
    with open(resource_path, encoding="utf-8") as f:
        return f.read()


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
@ignore_on_failure
def test_custom_qir_intrinsics_generates_qir():
    expected_qir = read_resource_file("custom_intrinsics.ll")

    def my_gate(self: QuantumCircuit, qubit: QubitSpecifier):
        return self.append(Gate(name="my_gate", num_qubits=1, params=[]), [qubit])

    QuantumCircuit.my_gate = my_gate

    class CustomTarget(QirTarget):
        def __init__(self):
            super().__init__()
            self.add_instruction(
                Gate(name="my_gate", num_qubits=1, params=[]), name="my_gate"
            )

    target = CustomTarget()
    circuit = QuantumCircuit(1, 1)
    circuit.my_gate(0)
    circuit.measure(0, 0)

    target_profile = TargetProfile.Adaptive_RI

    options = {
        "search_path": get_resource_path(),
        "includes": ("stdgates.inc", "custom_intrinsics.inc"),
    }

    backend = QSharpBackend(target_profile=target_profile, target=target)
    qir = backend.qir(circuit, **options)
    assert qir == expected_qir


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
@ignore_on_failure
def test_custom_qir_intrinsics_is_simulatable():
    def my_gate(self: QuantumCircuit, qubit: QubitSpecifier):
        return self.append(Gate(name="my_gate", num_qubits=1, params=[]), [qubit])

    QuantumCircuit.my_gate = my_gate

    class CustomTarget(QirTarget):
        def __init__(self):
            super().__init__()
            self.add_instruction(
                Gate(name="my_gate", num_qubits=1, params=[]), name="my_gate"
            )

    target = CustomTarget()
    circuit = QuantumCircuit(1, 1)
    circuit.my_gate(0)
    circuit.measure(0, 0)

    target_profile = TargetProfile.Adaptive_RI

    options = {
        "search_path": get_resource_path(),
        "includes": ("stdgates.inc", "custom_intrinsics.inc"),
    }

    backend = QSharpBackend(target_profile=target_profile, target=target)
    result = backend.run(circuit, **options).result()
    assert result.get_counts() == {"1": 1024}
