# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Shim exposing qsharp.estimator as qdk.estimator."""

from __future__ import annotations

try:  # pragma: no cover
    from qsharp.estimator import *  # type: ignore  # noqa: F401,F403
except Exception as ex:  # pragma: no cover
    raise ImportError(
        "qdk.estimator requires the 'qsharp' package with 'qsharp.estimator' available."
    ) from ex
