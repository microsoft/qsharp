# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Re-export shim for the optional widgets package as qdk.widgets.

If widgets is not installed (with the qdk[widgets] extra), importing this
module raises an ImportError describing how to enable it.
"""

from __future__ import annotations

try:
    from qsharp_widgets import *
except Exception as ex:
    raise ImportError(
        "qdk.widgets requires the widgets extra. Install with 'pip install qdk[widgets]'."
    ) from ex
