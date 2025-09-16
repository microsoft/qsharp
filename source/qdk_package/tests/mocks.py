# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Centralized mock helpers for tests.

Provides lightweight stand-ins for optional (and required during tests) dependencies.

Functions return a list of module names they created so callers can later clean them
up using cleanup_modules(). This keeps test intent explicit.
"""

from __future__ import annotations

import sys
import types
from typing import List


def mock_qsharp() -> List[str]:
    """Ensure a minimal 'qsharp' module exists.

    In real usage the qsharp package provides a compiled extension. Tests only
    need the attribute surface that qdk re-exports (run/estimate presently used
    for sanity checks). If the real package is installed this is a no-op.
    """
    created: List[str] = []
    if "qsharp" not in sys.modules:
        stub = types.ModuleType("qsharp")

        def _not_impl(*_a, **_k):  # pragma: no cover - placeholder
            raise NotImplementedError(
                "qsharp stub: real 'qsharp' package not installed"
            )

        stub.run = _not_impl  # type: ignore[attr-defined]
        stub.estimate = _not_impl  # type: ignore[attr-defined]
        # Provide utility symbols expected to re-export at root
        stub.code = object()  # type: ignore[attr-defined]
        stub.set_quantum_seed = lambda *_a, **_k: None  # type: ignore[attr-defined]
        stub.set_classical_seed = lambda *_a, **_k: None  # type: ignore[attr-defined]
        stub.dump_machine = lambda *_a, **_k: None  # type: ignore[attr-defined]
        stub.dump_circuit = lambda *_a, **_k: None  # type: ignore[attr-defined]

        class _T:  # placeholder types
            pass

        stub.Result = _T  # type: ignore[attr-defined]
        stub.TargetProfile = _T  # type: ignore[attr-defined]
        stub.StateDump = _T  # type: ignore[attr-defined]
        stub.ShotResult = _T  # type: ignore[attr-defined]
        stub.PauliNoise = _T  # type: ignore[attr-defined]
        stub.DepolarizingNoise = _T  # type: ignore[attr-defined]
        stub.BitFlipNoise = _T  # type: ignore[attr-defined]
        stub.PhaseFlipNoise = _T  # type: ignore[attr-defined]
        stub.__all__ = [
            "run",
            "estimate",
            "code",
            "set_quantum_seed",
            "set_classical_seed",
            "dump_machine",
            "dump_circuit",
            "Result",
            "TargetProfile",
            "StateDump",
            "ShotResult",
            "PauliNoise",
            "DepolarizingNoise",
            "BitFlipNoise",
            "PhaseFlipNoise",
        ]
        sys.modules["qsharp"] = stub
        created.append("qsharp")
    return created


def mock_widgets() -> List[str]:
    created: List[str] = []
    if "qsharp_widgets" not in sys.modules:
        mod = types.ModuleType("qsharp_widgets")
        mod.__version__ = "1.20.0-mock"
        sys.modules["qsharp_widgets"] = mod
        created.append("qsharp_widgets")
    return created


def mock_azure() -> List[str]:
    created: List[str] = []
    if "azure" not in sys.modules:
        sys.modules["azure"] = types.ModuleType("azure")
        created.append("azure")
    if "azure.quantum" not in sys.modules:
        aq = types.ModuleType("azure.quantum")
        aq.__version__ = "3.2.0-mock"
        sys.modules["azure.quantum"] = aq
        created.append("azure.quantum")
    return created


def mock_qiskit() -> List[str]:
    created: List[str] = []
    if "qiskit" not in sys.modules:
        qk = types.ModuleType("qiskit")
        qk.__version__ = "1.2.2-mock"

        def transpile(*circuits, **_kwargs):  # pragma: no cover - placeholder
            return {"circuits": len(circuits)}

        qk.transpile = transpile  # type: ignore[attr-defined]
        sys.modules["qiskit"] = qk
        created.append("qiskit")
    return created


def cleanup_modules(created: List[str]) -> None:
    """Remove synthetic modules created during a test if still present."""
    for name in created:
        sys.modules.pop(name, None)
    if (
        "azure" in created
        and getattr(sys.modules.get("azure"), "__file__", None) is None
    ):
        sys.modules.pop("azure", None)
