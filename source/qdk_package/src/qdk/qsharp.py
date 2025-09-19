# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Re-export of `qsharp` as `qdk.qsharp`.

If `qsharp` isn't installed this raises an ImportError.
"""

from __future__ import annotations

try:
    from qsharp import *
except Exception as ex:
    raise ImportError(
        "qdk.qsharp requires the 'qsharp' package. Install with 'pip install qsharp'."
    ) from ex
