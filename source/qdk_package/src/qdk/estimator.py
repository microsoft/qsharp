# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Shim exposing qsharp.estimator as qdk.estimator."""

from __future__ import annotations

try:
    from qsharp.estimator import *
except Exception as ex:
    raise ImportError(
        "qdk.estimator requires the 'qsharp' package with 'qsharp.estimator' available."
    ) from ex
