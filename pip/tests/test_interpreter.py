# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qsharp._native import (Interpreter, Result, Pauli)


# Tests for the native Q# interpreter class


def test_output() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('Message("Hello, world!")')
    assert out[0].__repr__() == "Hello, world!"
    assert value is None
    assert len(err) == 0


def test_dump_output() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("""
    use q1 = Qubit();
    use q2 = Qubit();
    X(q1);
    Microsoft.Quantum.Diagnostics.DumpMachine();
    ResetAll([q1, q2]);
    """)
    assert out[0].__repr__() == "STATE:\n|01⟩: 1.0000+0.0000i"


def test_error() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("a864")
    assert len(err) == 1
    assert err[0].message == f"name error"


def test_multiple_statements() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("1; Zero")
    assert value == Result.Zero


def test_value_int() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("5")
    assert value == 5


def test_value_double() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("3.1")
    assert value == 3.1


def test_value_bool() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret("true")
    assert value == True


def test_value_string() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('"hello"')
    assert value == "hello"


def test_value_result() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('One')
    assert value == Result.One


def test_value_pauli() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('PauliX')
    assert value == Pauli.X


def test_value_tuple() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('(1, "hello", One)')
    assert value == (1, "hello", Result.One)


def test_value_unit() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('Unit')
    assert value is None


def test_value_array() -> None:
    e = Interpreter()
    (value, out, err) = e.interpret('[1, 2, 3]')
    assert value == [1, 2, 3]
