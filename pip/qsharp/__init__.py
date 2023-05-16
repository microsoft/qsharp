# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import interpret, interpret_file

from ._native import Result, Pauli, QSharpError

# IPython notebook specific features
try:
    if __IPYTHON__:  # type: ignore
        from ._ipython import register_magic, enable_classic_notebook_codemirror_mode

        register_magic()
        enable_classic_notebook_codemirror_mode()
except NameError:
    pass


__all__ = ["interpret", "interpret_file", "Result", "Pauli", "QSharpError"]
