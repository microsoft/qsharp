# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import subprocess
from ._native import Interpreter, Target


def init(target: Target = Target.Full) -> None:
    """
    Initializes the Q# interpreter.

    :param target: The target to use for the Q# interpreter.
    """
    global _interpreter
    _interpreter = Interpreter(target)


def get_interpreter() -> Interpreter:
    """
    Returns the Q# interpreter.

    :returns: The Q# interpreter.
    """
    global _interpreter
    return _interpreter


def interpret(input):
    """
    Interprets Q# source code.

    Output is printed to console.

    :param input: The Q# source code to interpret.
    :returns value: The value returned by the last statement in the input.
    :raises QSharpError: If there is an error interpreting the input.
    """

    def callback(output):
        print(output)

    return _interpreter.interpret(input, callback)


def interpret_file(path) -> None:
    """
    Reads Q# source code from a file and interprets it.

    :param path: The path to the Q# source file.
    :returns: The value returned by the last statement in the line.
    :raises: QSharpError
    """
    f = open(path, mode="r", encoding="utf-8")
    return interpret(f.read())


def compile(entry_expr):
    """
    Compiles Q# into a job for submission to Azure
    """
    ll_str = _interpreter.qir(entry_expr)
    return QirWrapper("main", ll_str)


interim_ll_path = "C:\\temp\\interim.ll"
interim_bc_path = "C:\\temp\\interim.bc"


class QirWrapper:
    # Signature important for azure quantum
    _name: str

    def __init__(self, name: str, ll_str: str):
        self._name = name
        # Write ll to file
        file = open(interim_ll_path, "w")
        file.write(ll_str)
        file.close()
        # Make .bc
        child = subprocess.Popen(
            [
                "C:\\temp\\qat.exe",
                "--apply",
                "--always-inline",
                "--no-disable-record-output-support",
                "--entry-point-attr",
                "entry_point",
                interim_ll_path,
                "C:\\src\\qsharp\\compiler\\qsc_codegen\\src\\qir_base\\decomp.ll",
                "-o",
                interim_bc_path,
            ]
        )
        if child.wait() != 0:
            raise Exception(f"Linking failed: '{child.returncode}'")
        bc_file = open(interim_bc_path, "rb")
        bc = bc_file.read()
        self.bc = bc

    # Signature important for azure quantum
    def _repr_qir_(self, **kwargs) -> bytes:
        return self.bc
