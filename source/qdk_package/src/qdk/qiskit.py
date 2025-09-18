# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Re-export shim for the optional qiskit package as qdk.qiskit.

If qiskit is not installed (with the qdk[qiskit] extra), importing this
module raises an ImportError describing how to enable it.
"""

from __future__ import annotations

try:  # pragma: no cover - presence check
    from qiskit import *  # type: ignore  # noqa: F401,F403
    from qsharp.interop.qiskit import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.qiskit requires the qiskit extra. Install with 'pip install qdk[qiskit]'."
    ) from ex
