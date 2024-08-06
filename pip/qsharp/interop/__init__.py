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
        QSharpSimulator,
        QsJob,
        ReJob,
        ReSimulator,
        SynchronousExecutor,
    )

    __all__ = [
        "QSharpSimulator",
        "QsJob",
        "ReJob",
        "ReSimulator",
        "SynchronousExecutor",
    ]
else:
    __all__ = []
