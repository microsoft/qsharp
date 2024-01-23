# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import (
    init,
    eval,
    run,
    compile,
    estimate,
    set_quantum_seed,
    set_classical_seed,
    dump_machine,
)

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
    "run",
    "set_quantum_seed",
    "set_classical_seed",
    "dump_machine",
    "compile",
    "estimate",
    "Result",
    "Pauli",
    "QSharpError",
    "TargetProfile",
    "StateDump",
]
