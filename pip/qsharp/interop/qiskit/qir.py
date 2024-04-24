# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Optional
from qiskit import QuantumCircuit
from qsharp import TargetProfile

from .backends import QSharpSimulator


def convert_qiskit_to_qir(
    circuit: QuantumCircuit,
    target_profile: TargetProfile = TargetProfile.Base,
    entry_expr: Optional[str] = None,
    search_path: Optional[str] = None,
    **kwargs,
) -> str:
    """
    Converts a Qiskit QuantumCircuit to QIR (Quantum Intermediate Representation).

    Args:
        circuit ('QuantumCircuit'): The input Qiskit QuantumCircuit object.
        target_profile (TargetProfile, optional): The target profile for the backend. Defaults to TargetProfile.Base.
        entry_expr (str, optional): The entry expression for the QIR conversion. Defaults to None.
        search_path (str, optional): The search path for the backend. Defaults to '.'.

    Returns:
        str: The converted QIR code as a string.
    """
    from .utils import _convert_qiskit_to_qasm3, _convert_qasm3_to_qir

    backend = QSharpSimulator(target_profile=target_profile, **kwargs)
    qasm3_source = _convert_qiskit_to_qasm3(circuit, backend, **kwargs)
    return _convert_qasm3_to_qir(
        qasm3_source, circuit.name, target_profile, entry_expr, search_path
    )
