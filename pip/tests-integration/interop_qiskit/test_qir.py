# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest
from qsharp import TargetProfile, QSharpError

from . import QISKIT_AVAILABLE, SKIP_REASON


if QISKIT_AVAILABLE:
    from .test_circuits import (
        core_tests,
        generate_repro_information,
    )
    from qsharp.interop.qiskit import QasmError
    from qiskit.circuit import QuantumCircuit, Parameter
    from qsharp.interop.qiskit.backends import QSharpSimulator
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

    try:
        backend = QSharpSimulator(target_profile=target_profile)
        qir = backend.qir(circuit)
        assert qir is not None
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_with_unbound_param_raises():
    theta = Parameter("theta")

    circuit = QuantumCircuit(1)
    circuit.name = "test"
    circuit.rx(theta, 0)
    circuit.measure_all()

    target_profile = TargetProfile.Base

    try:
        with pytest.raises(QSharpError) as ex:
            backend = QSharpSimulator(target_profile=target_profile)
            _ = backend.qir(circuit)
        message = str(ex)
        assert "entry point could not compiled." in message
        assert "Qsc.TypeCk.TyMismatch" in message
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_with_bound_param_with_wrong_type_raises():
    theta = Parameter("theta")

    circuit = QuantumCircuit(1)
    circuit.name = "test"
    circuit.rx(theta, 0)
    circuit.measure_all()

    target_profile = TargetProfile.Base

    try:
        with pytest.raises(QSharpError) as ex:
            backend = QSharpSimulator(target_profile=target_profile)
            _ = backend.qir(circuit, entry_expr="qasm3_import.test(true)")
        message = str(ex)
        assert "entry point could not compiled." in message
        assert "Qsc.TypeCk.TyMismatch" in message
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_with_bound_param():
    theta = Parameter("theta")

    circuit = QuantumCircuit(1)
    circuit.name = "test"
    circuit.rx(theta, 0)
    circuit.measure_all()

    target_profile = TargetProfile.Base

    try:
        backend = QSharpSimulator(target_profile=target_profile)
        qir = backend.qir(circuit, entry_expr="qasm3_import.test(0.5)")
        assert qir is not None
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_without_measuring_into_all_registers_raises():
    circuit = QuantumCircuit(2, 2)
    circuit.rx(0.5, 0)

    target_profile = TargetProfile.Base

    try:
        with pytest.raises(QSharpError) as ex:
            backend = QSharpSimulator(target_profile=target_profile)
            _ = backend.qir(circuit)
        assert "ensure all output registers have been measured into." in str(ex)
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
def test_generating_qir_without_registers_raises():
    circuit = QuantumCircuit(2)
    circuit.rx(0.5, 0)

    target_profile = TargetProfile.Base

    try:
        with pytest.raises(QasmError) as ex:
            backend = QSharpSimulator(target_profile=target_profile)
            _ = backend.qir(circuit)
        assert "Qiskit circuits must have output registers." in str(ex)
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex
