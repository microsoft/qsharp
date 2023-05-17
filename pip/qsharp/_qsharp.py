# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Interpreter

# Create a Q# interpreter singleton.
_interpreter = Interpreter()


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
