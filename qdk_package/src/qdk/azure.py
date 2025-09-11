"""Re-export convenience shim for the optional azure-quantum package as qdk.azure.

If azure-quantum (version pinned via extra) is not installed, importing this
module raises an ImportError instructing how to enable the extra.
"""

from __future__ import annotations

try:  # pragma: no cover - simple presence check
    # azure-quantum's public surface is under azure.quantum.*; we import the top package
    from azure.quantum import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.azure requires the azure extra. Install with 'pip install qdk[azure]'."
    ) from ex
