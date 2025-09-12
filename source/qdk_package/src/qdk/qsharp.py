"""Re-export of `qsharp` as `qdk.qsharp`.

If `qsharp` isn't installed this raises an ImportError.
"""

from __future__ import annotations

try:  # pragma: no cover - simple presence check
    from qsharp import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.qsharp requires the 'qsharp' package. Install with 'pip install qsharp'."
    ) from ex
