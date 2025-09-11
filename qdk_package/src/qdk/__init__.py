"""qdk bundling meta-package.

Design goals:
    * Provide a single import root `qdk` that exposes bundled quantum tooling as
        submodules (`qdk.qsharp`, `qdk.widgets`, future optional extras similarly).
    * Keep direct symbol re-export minimal; users import submodules instead of symbols.

Optional extras:
    widgets -> installs `qsharp-widgets`, available as `qdk.widgets`.
    azure   -> installs `azure-quantum`, available as `qdk.azure`.

"""

from __future__ import annotations

from importlib import import_module
from typing import Dict, List

# Always make the underlying qsharp package available as submodule `qdk.qsharp`.
try:  # pragma: no cover
    import_module("qsharp")
except ModuleNotFoundError as _ex:  # pragma: no cover
    raise ImportError(
        "qdk requires the 'qsharp' package. Install with 'pip install qsharp'."
    ) from _ex

# Import our local re-export shim (created separately) so `qdk.qsharp` works.
try:  # pragma: no cover
    from . import qsharp as qsharp  # type: ignore  # noqa: F401
except Exception as _ex:  # pragma: no cover
    raise

# Public API surface (kept intentionally small)
__all__ = ["qsharp", "widgets_available", "azure_available", "require"]


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


def require(feature: str):
    """Return the module backing a named feature or raise ImportError.

    Recognized features:
        * 'qsharp'  -> qdk.qsharp
        * 'widgets' -> qdk.widgets (requires widgets extra)
        * 'azure'   -> qdk.azure  (requires azure extra)
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
    raise ImportError(
        f"Feature '{feature}' is not recognized. Available: qsharp, widgets, azure (if installed)."
    )
