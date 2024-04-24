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
        convert_qiskit_to_qir,
        QsJob,
        ReJob,
        ReSimulator,
        QSharpSimulator,
    )

    __all__ = [
        "convert_qiskit_to_qir",
        "QsJob",
        "ReJob",
        "ReSimulator",
        "QSharpSimulator",
    ]
else:
    __all__ = []
