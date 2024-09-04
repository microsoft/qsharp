# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.


def _test_qiskit_import() -> bool:
    try:
        import qiskit

        return True
    except ImportError:
        return False


if _test_qiskit_import():
    from .qiskit import (
        QasmError,
        QSharpBackend,
        QsJob,
        ReJob,
        ResourceEstimatorBackend,
        DetaultExecutor,
    )

    __all__ = [
        "QasmError",
        "QSharpBackend",
        "QsJob",
        "ReJob",
        "ResourceEstimatorBackend",
        "DefaultExecutor",
    ]
else:
    __all__ = []
