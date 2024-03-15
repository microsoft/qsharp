# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import (
    init,
    eval,
    run,
    compile,
    circuit,
    estimate,
    set_quantum_seed,
    set_classical_seed,
    dump_machine,
    dump_circuit,
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
    "dump_circuit",
    "compile",
    "circuit",
    "estimate",
    "Result",
    "Pauli",
    "QSharpError",
    "TargetProfile",
    "StateDump",
]
