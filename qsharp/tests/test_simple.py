# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qsharp import Evaluator


def test_int_lit() -> None:
    e = Evaluator()
    (value, out, err) = e.eval("5")
    assert value == "5"


def test_output_follows() -> None:
    e = Evaluator()
    (value, out, err) = e.eval('Message("Hello, world!")')
    assert out == "Hello, world!\n"


def test_unknown_ident() -> None:
    e = Evaluator()
    expr = "a864"
    (value, out, err) = e.eval(expr)
    assert err == f"`{expr}` not found in this scope"
