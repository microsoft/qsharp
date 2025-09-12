"""Re-export shim for the optional widgets package as qdk.widgets.

If widgets is not installed (with the qdk[widgets] extra), importing this
module raises an ImportError describing how to enable it.
"""

from __future__ import annotations

try:  # pragma: no cover - minimal wrapper
    from qsharp_widgets import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.widgets requires the widgets extra. Install with 'pip install qdk[widgets]'."
    ) from ex
