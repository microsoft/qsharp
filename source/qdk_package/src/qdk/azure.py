# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Re-export shim for the optional azure-quantum package as qdk.azure.

If azure is not installed (with the qdk[azure] extra), importing this
module raises an ImportError describing how to enable it.
"""

from __future__ import annotations

try:  # pragma: no cover - simple presence check
    # azure-quantum's public surface is under azure.quantum.*; we import the top package
    from azure.quantum import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.azure requires the azure extra. Install with 'pip install qdk[azure]'."
    ) from ex
