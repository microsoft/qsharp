# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Interpreter

# Create a Q# interpreter singleton.
interpreter = Interpreter()


def interpret(expr):
    """
    Interprets a Q# expression.
    Returns the result.
    Prints any output to stderr.
    Throws RuntimeException or CompilationException
    """
    (value, outputs) = _interpret_with_outputs(expr)

    for output in outputs:
        print(output)

    return value


def _interpret_with_outputs(expr):
    (value, outputs, err) = interpreter.interpret(expr)

    # TODO: The interpreter will be updated to return a single
    # compilation/runtime error, so this will be unnecessary.
    for error in err:
        if error.error_type == "CompilationError":
            raise CompilationException(err)
        else:
            raise RuntimeException(err)

    return (value, outputs)


def interpret_file(path) -> None:
    f = open(path, mode="r", encoding="utf-8")
    return interpret(f.read())


class QSharpException(Exception):
    def __init__(self, diagnostics):
        self.diagnostics = diagnostics


class CompilationException(QSharpException):
    pass


class RuntimeException(QSharpException):
    pass
