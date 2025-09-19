# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Shim exposing qsharp.openqasm as qdk.openqasm."""

from __future__ import annotations

try:
    from qsharp.openqasm import *
except Exception as ex:
    raise ImportError(
        "qdk.openqasm requires the 'qsharp' package with 'qsharp.openqasm' available."
    ) from ex
