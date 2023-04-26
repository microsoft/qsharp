# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Interpreter

# Create a Q# interpreter singleton.
interpreter = Interpreter()


def interpret(expr):
    """
    Interprets a line of Q# source code.

    :param expr: The Q# source code to interpret.
    :returns: The value returned by the last statement in the line.
    :raises: CompilationException, RuntimeException
    """
    (value, outputs) = _interpret_with_outputs(expr)

    for output in outputs:
        print(output)

    return value


def _interpret_with_outputs(expr):
    (value, outputs, err) = interpreter.interpret(expr)

    # TODO(minestarks): The interpreter will be updated to return a single
    # compilation/runtime error, so this will be unnecessary.
    for error in err:
        if error.error_type == "CompilationError":
            raise CompilationException(err)
        else:
            raise RuntimeException(err)

    return (value, outputs)


def interpret_file(path) -> None:
    """
    Reads Q# source code from a file and interprets it.

    :param path: The path to the Q# source file.
    :returns: The value returned by the last statement in the line.
    :raises: CompilationException, RuntimeException, OSError
    """
    f = open(path, mode="r", encoding="utf-8")
    return interpret(f.read())


class QSharpException(Exception):
    def __init__(self, diagnostics):
        self.diagnostics = diagnostics


class CompilationException(QSharpException):
    pass


class RuntimeException(QSharpException):
    pass
