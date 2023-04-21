# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qsharp._native import Interpreter


def test_int_lit() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("5")
    assert value == 5


def test_output_follows() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('Message("Hello, world!")')
    assert out[0].__repr__() == "Hello, world!"


def test_unknown_ident() -> None:
    e = Interpreter()
    expr = "a864"
    (value, out, err) = e.interpret(expr)
    assert len(err) == 1
    assert err[0].message == f"`{expr}` not found in this scope"
