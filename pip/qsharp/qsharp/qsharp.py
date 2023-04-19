# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Evaluator
import sys

# debug output to stderr


def print_dbg(str, **kwargs):
#    print("\x1b[2m<debug>: " + str.replace("\n", "\n<debug>: ") + "\x1b[0m", file=sys.stderr, **kwargs)
    pass


# Create a Q# evaluator.
# TODO: Q: Should this be a singleton or should we allow multiple prgograms?
evaluator = Evaluator()

# TODO: Many shots


def interpret(expr):
    """
    Interprets a Q# expression.
    Returns the result.
    Prints any output to stderr.
    Throws RuntimeException or CompilationException
    """
    (value, outputs) = interpret_with_dumps(expr)

    for output in outputs:
        print(output)

    return value


def interpret_with_dumps(expr):
    (value, outputs, err) = evaluator.eval(expr)

    print_dbg(f"value: {value} (type: {type(value).__name__})")
    print_dbg(f"out: {outputs}")
    print_dbg(f"err: {err}")

    # iterate over the list err, and throw CompilationException if any of the errors are of type CompilationError
    # TODO: Multiple compilationerrors, handle more gracefully. Also, do we really need multiple exception types?
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
