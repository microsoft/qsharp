"""Re-export shim for the optional qsharp-jupyterlab package as qdk.jupyterlab.

If qsharp-jupyterlab is not installed (with the qdk[jupyterlab] extra), importing this
module raises an ImportError describing how to enable it.
"""

from __future__ import annotations

try:  # pragma: no cover - presence check only
    from qsharp_jupyterlab import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.jupyterlab requires the jupyterlab extra. Install with 'pip install qdk[jupyterlab]'."
    ) from ex
