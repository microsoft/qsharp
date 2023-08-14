# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Interpreter, Target

_interpreter = None


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
    if _interpreter is None:
        raise RuntimeError(
            "Q# interpreter not initialized. Call qsharp.init() with any desired configuration settings first."
        )
    return _interpreter


def eval(source):
    """
    Evaluates Q# source code.

    Output is printed to console.

    :param source: The Q# source code to evaluate.
    :returns value: The value returned by the last statement in the source code.
    :raises QSharpError: If there is an error evaluating the source code.
    """

    def callback(output):
        print(output)

    return get_interpreter().interpret(source, callback)


def eval_file(path) -> None:
    """
    Reads Q# source code from a file and evaluates it.

    :param path: The path to the Q# source file.
    :returns: The value returned by the last statement in the file.
    :raises: QSharpError
    """
    f = open(path, mode="r", encoding="utf-8")
    return eval(f.read())


def compile(entry_expr):
    """
    Compiles Q# into a job for submission to Azure
    """
    ll_str = get_interpreter().qir(entry_expr)
    return QirWrapper("main", ll_str)


class QirWrapper:
    # Signature important for azure quantum
    _name: str

    def __init__(self, name: str, ll_str: str):
        self._name = name
        self._ll_str = ll_str

    # Signature important for azure quantum
    def _repr_qir_(self, **kwargs) -> bytes:
        return self._ll_str.encode("utf-8")
