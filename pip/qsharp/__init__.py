# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import (
    interpret,
    interpret_file,
    QSharpException,
    CompilationException,
    RuntimeException
)

from ._native import (
    Result,
    Pauli
)

# Register the IPython magic if we're in an IPython session
try:
    if __IPYTHON__: # type: ignore
        from ._ipython import register_magic
        register_magic()
except NameError:
    pass


__all__ = [
    "interpret",
    "interpret_file",
    "Result",
    "Pauli",
    "QSharpException",
    "CompilationException",
    "RuntimeException",
]
