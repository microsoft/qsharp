# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""qdk bundling meta-package.

Design goals:
    * Provide a single import root `qdk` that exposes bundled quantum tooling as
        submodules (`qdk.qsharp`, `qdk.widgets`, etc.).

Optional extras:
    widgets -> installs `qsharp-widgets`, available as `qdk.widgets`.
    azure   -> installs `azure-quantum`, available as `qdk.azure`.
    qiskit  -> installs `qiskit`, available as `qdk.qiskit`.
    jupyterlab -> installs `qsharp-jupyterlab`, available as `qdk.jupyterlab`.

"""

from __future__ import annotations

from importlib import import_module

# Eagerly check for qsharp presence to provide a clear error message.
try:  # pragma: no cover
    import_module("qsharp")
except ModuleNotFoundError as _ex:  # pragma: no cover
    raise ImportError(
        "qdk requires the 'qsharp' package. Install with 'pip install qsharp'."
    ) from _ex

# Eagerly import qdk.qsharp to ensure qdk.qsharp is always available if qdk is.
try:  # pragma: no cover
    from . import qsharp as qsharp  # type: ignore  # noqa: F401
except Exception as _ex:  # pragma: no cover
    raise

# Some common utilities are lifted to the qdk root.
from qsharp import code  # type: ignore  # noqa: F401
from qsharp import (
    set_quantum_seed,
    set_classical_seed,
    dump_machine,
    dump_circuit,
    Result,
    TargetProfile,
    StateDump,
    ShotResult,
    PauliNoise,
    DepolarizingNoise,
    BitFlipNoise,
    PhaseFlipNoise,
)

__all__ = [
    "qsharp",
    "widgets_available",
    "azure_available",
    "qiskit_available",
    "jupyterlab_available",
    "require",
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


def widgets_available() -> bool:
    """Return True if the widgets extra is installed."""
    try:  # pragma: no cover
        import_module("qsharp_widgets")
        return True
    except Exception:
        return False


def azure_available() -> bool:
    """Return True if the azure extra (azure-quantum) is installed."""
    try:  # pragma: no cover
        import_module(
            "azure.quantum"
        )  # azure-quantum installs namespace under azure.quantum
        return True
    except Exception:
        return False


def qiskit_available() -> bool:
    """Return True if the qiskit extra is installed."""
    try:  # pragma: no cover
        import_module("qiskit")
        return True
    except Exception:
        return False


def jupyterlab_available() -> bool:
    """Return True if the jupyterlab extra (qsharp-jupyterlab) is installed."""
    try:  # pragma: no cover
        import_module("qsharp_jupyterlab")
        return True
    except Exception:
        return False


def require(feature: str):
    """Return the module backing a named feature or raise ImportError.

    Recognized features:
        * 'qsharp'  -> qdk.qsharp
        * 'widgets' -> qdk.widgets (requires widgets extra)
        * 'azure'   -> qdk.azure  (requires azure extra)
        * 'qiskit'  -> qdk.qiskit (requires qiskit extra)
        * 'jupyterlab' -> qdk.jupyterlab (requires jupyterlab extra)
    """
    if feature == "qsharp":
        return qsharp
    if feature == "widgets":
        if not widgets_available():
            raise ImportError(
                "Feature 'widgets' unavailable. Install with 'pip install qdk[widgets]'."
            )
        return import_module("qdk.widgets")
    if feature == "azure":
        if not azure_available():
            raise ImportError(
                "Feature 'azure' unavailable. Install with 'pip install qdk[azure]'."
            )
        return import_module("qdk.azure")
    if feature == "qiskit":
        if not qiskit_available():
            raise ImportError(
                "Feature 'qiskit' unavailable. Install with 'pip install qdk[qiskit]'."
            )
        return import_module("qdk.qiskit")
    if feature == "jupyterlab":
        if not jupyterlab_available():
            raise ImportError(
                "Feature 'jupyterlab' unavailable. Install with 'pip install qdk[jupyterlab]'."
            )
        return import_module("qdk.jupyterlab")
    raise ImportError(
        f"Feature '{feature}' is not recognized. Available: qsharp, widgets, azure, qiskit, jupyterlab (if installed)."
    )
