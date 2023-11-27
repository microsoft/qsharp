# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import init, eval, eval_file, run, compile, dump_machine

from ._native import Result, Pauli, QSharpError, TargetProfile, StateDump

# IPython notebook specific features
try:
    if __IPYTHON__:  # type: ignore
        from ._ipython import register_magic, enable_classic_notebook_codemirror_mode

        register_magic()
        enable_classic_notebook_codemirror_mode()
except NameError:
    pass


__all__ = [
    "init",
    "eval",
    "eval_file",
    "run",
    "dump_machine",
    "compile",
    "Result",
    "Pauli",
    "QSharpError",
    "TargetProfile",
    "StateDump"
]
