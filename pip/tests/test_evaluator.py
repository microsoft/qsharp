# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qsharp import Evaluator


def test_int_lit() -> None:
    e = Evaluator()
    (value, out, err) = e.eval("5")
    assert value == 5


def test_output_follows() -> None:
    e = Evaluator()
    (value, out, err) = e.eval('Message("Hello, world!")')
    assert out[0].__repr__() == "Hello, world!"


def test_unknown_ident() -> None:
    e = Evaluator()
    expr = "a864"
    (value, out, err) = e.eval(expr)
    assert len(err) == 1
    assert err[0].message == f"`{expr}` not found in this scope"
