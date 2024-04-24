# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest
from qsharp import TargetProfile

from .. import QISKIT_AVAILABLE

if QISKIT_AVAILABLE:
    from qiskit.circuit.random import random_circuit
    from qsharp.interop import QSharpBackend


def _generate_random_fixture(
    num_qubits: int, depth: int, target_profile: TargetProfile, name: str
):
    @pytest.fixture()
    def random():
        if target_profile == TargetProfile.Base:
            circuit = random_circuit(num_qubits, depth, measure=True, reset=True)
        elif target_profile == TargetProfile.Adaptive_RI:
            circuit = random_circuit(
                num_qubits, depth, measure=True, conditional=True, reset=True
            )
        else:
            raise ValueError(f"Unsupported QIR profile: {target_profile}")

        backend = QSharpBackend(target_profile=target_profile)
        circuit = backend.transpile(circuit)
        circuit.name = name
        return circuit

    return random


# Generate random fixtures
random_fixtures = []
random_fixtures_small = []


def get_name(num_qubits: int, depth: int, target_profile: TargetProfile, prefix: str):
    if target_profile == TargetProfile.Base:
        return f"random_{prefix}_{num_qubits}x{depth}_base"
    else:
        return f"random_{prefix}_{num_qubits}x{depth}_adaptive_ri"


if QISKIT_AVAILABLE:
    for num_qubits, depth in [(i, j) for i in range(2, 11) for j in range(2, 11)]:
        for target_profile in [TargetProfile.Base, TargetProfile.Adaptive_RI]:
            name = get_name(num_qubits, depth, target_profile, "full")
            fixture = _generate_random_fixture(num_qubits, depth, target_profile, name)
            locals()[name] = fixture
            random_fixtures.append(name)

    for num_qubits, depth in [(i, j) for i in range(2, 5) for j in range(2, 5)]:
        for target_profile in [TargetProfile.Base, TargetProfile.Adaptive_RI]:
            name = get_name(num_qubits, depth, target_profile, "small")
            fixture = _generate_random_fixture(num_qubits, depth, target_profile, name)
            locals()[name] = fixture
            random_fixtures_small.append(name)
