# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Interpreter

# Create a Q# interpreter singleton.
_interpreter = Interpreter()


def eval(source):
    """
    Evaluates Q# source code.

    Output is printed to console.

    :param input: The Q# source code to evaluate.
    :returns value: The value returned by the last statement in the input.
    :raises QSharpError: If there is an error evaluating the input.
    """

    def callback(output):
        print(output)

    return _interpreter.interpret(source, callback)


def eval_file(path) -> None:
    """
    Reads Q# source code from a file and evaluates it.

    :param path: The path to the Q# source file.
    :returns: The value returned by the last statement in the line.
    :raises: QSharpError
    """
    f = open(path, mode="r", encoding="utf-8")
    return eval(f.read())
