"""Re-export of `qsharp_widgets` as `qdk.widgets`.

If `qsharp_widgets` is not installed, importing this module raises an
ImportError instructing how to enable the extra.
"""

from __future__ import annotations

try:  # pragma: no cover - minimal wrapper
    from qsharp_widgets import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.widgets requires the widgets extra. Install with 'pip install qdk[widgets]'."
    ) from ex
