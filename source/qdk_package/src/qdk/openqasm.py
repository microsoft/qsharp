# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Shim exposing qsharp.openqasm as qdk.openqasm."""

from __future__ import annotations

try:  # pragma: no cover
    from qsharp.openqasm import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.openqasm requires the 'qsharp' package with 'qsharp.openqasm' available."
    ) from ex
