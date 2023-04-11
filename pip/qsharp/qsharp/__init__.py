# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qsharp.qsharp import (
    interpret,
    interpret_file,
    interpret_with_dumps,
    QSharpException,
    CompilationException,
    RuntimeException
)

from qsharp._native import (
    Evaluator, # Temporarily keeping this for tests
    Result,
    Pauli
)

# If using IPython, forward some useful IQ# magic commands as IPython magic
# commands and define a couple new magic commands for IPython.
try:
    if __IPYTHON__:
        import qsharp.ipython
        qsharp.ipython.register_magic()
except NameError:
    pass


__all__ = [
    "interpret",
    "interpret_file",
    "interpret_with_dumps",
    "Result",
    "Pauli",
    "QSharpException",
    "CompilationException",
    "RuntimeException",
]