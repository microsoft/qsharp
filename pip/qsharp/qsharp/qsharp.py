# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Evaluator
import re

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
    (value, out, dumps) = interpret_with_dumps(expr)

    # TODO: Don't trim the string - have the evaluator not send so many newlines
    if out.strip():
        print(out.strip())

    return value


def interpret_with_dumps(expr):
    (value, out, err) = evaluator.eval(expr)

    print_dbg(f"value: {value} (type: {type(value).__name__})")
    print_dbg(f"out: {out}")
    print_dbg(f"err: {err}")

    dumps = parse_dump_string(out)

    # iterate over the list err, and throw CompilationException if any of the errors are of type CompilationError
    # TODO: Multiple compilationerrors, handle more gracefully. Also, do we really need multiple exception types?
    for error in err:
        if error.error_type == "CompilationError":
            raise CompilationException(err)
        else:
            raise RuntimeException(err)

    return (value, out, dumps)


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


# Quick and dirty output parser
def parse_dump_string(input_string):
    dumps = []
    matches = re.findall(re.compile(
        r'STATE:\n((^\|.+?⟩: .+?[+\-].+?i\n?)+)', re.MULTILINE), input_string)
    for match in matches:
        state_string = match[0]
        entries = state_string.split('\n')
        state = []
        for entry_string in entries:
            if (entry_string):
                entry_matches = re.findall(
                    r'\|(.+?)⟩: ((.+?)([+\-].+?)i)\n?', entry_string)
                entry_match = entry_matches[0]
                id = entry_match[0]
                r = float(entry_match[2])
                i = float(entry_match[3].replace("+-", "-"))
                c = complex(r, i)
                state.append((id, r, i, c))
        dumps.append(state)
    return dumps
