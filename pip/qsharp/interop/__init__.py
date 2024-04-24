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
        QiskitError,
        QSharpSimulator,
        QsJob,
        ReJob,
        ReSimulator,
        DetaultExecutor,
    )

    __all__ = [
        "QasmError",
        "QiskitError",
        "QSharpSimulator",
        "QsJob",
        "ReJob",
        "ReSimulator",
        "DetaultExecutor",
    ]
else:
    __all__ = []
