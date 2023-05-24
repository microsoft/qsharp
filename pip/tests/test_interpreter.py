# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from qsharp._native import Interpreter, Result, Pauli, QSharpError
import pytest


# Tests for the native Q# interpreter class


def test_output() -> None:
    e = Interpreter()

    def callback(output):
        nonlocal called
        called = True
        assert output.__repr__() == "Hello, world!"

    called = False
    value = e.interpret('Message("Hello, world!")', callback)
    assert called


def test_dump_output() -> None:
    e = Interpreter()

    def callback(output):
        nonlocal called
        called = True
        assert output.__repr__() == "STATE:\n|01⟩: 1.0000+0.0000𝑖"

    called = False
    value = e.interpret(
        """
    use q1 = Qubit();
    use q2 = Qubit();
    X(q1);
    Microsoft.Quantum.Diagnostics.DumpMachine();
    ResetAll([q1, q2]);
    """,
        callback,
    )
    assert called


def test_error() -> None:
    e = Interpreter()

    with pytest.raises(QSharpError) as excinfo:
        e.interpret("a864")
    assert str(excinfo.value).find("name error") != -1


def test_multiple_errors() -> None:
    e = Interpreter()

    with pytest.raises(QSharpError) as excinfo:
        e.interpret("operation Foo() : Unit { Bar(); Baz(); }")
    assert str(excinfo.value).find("`Bar` not found in this scope") != -1
    assert str(excinfo.value).find("`Baz` not found in this scope") != -1


def test_multiple_statements() -> None:
    e = Interpreter()
    value = e.interpret("1; Zero")
    assert value == Result.Zero


def test_value_int() -> None:
    e = Interpreter()
    value = e.interpret("5")
    assert value == 5


def test_value_double() -> None:
    e = Interpreter()
    value = e.interpret("3.1")
    assert value == 3.1


def test_value_bool() -> None:
    e = Interpreter()
    value = e.interpret("true")
    assert value == True


def test_value_string() -> None:
    e = Interpreter()
    value = e.interpret('"hello"')
    assert value == "hello"


def test_value_result() -> None:
    e = Interpreter()
    value = e.interpret("One")
    assert value == Result.One


def test_value_pauli() -> None:
    e = Interpreter()
    value = e.interpret("PauliX")
    assert value == Pauli.X


def test_value_tuple() -> None:
    e = Interpreter()
    value = e.interpret('(1, "hello", One)')
    assert value == (1, "hello", Result.One)


def test_value_unit() -> None:
    e = Interpreter()
    value = e.interpret("()")
    assert value is None


def test_value_array() -> None:
    e = Interpreter()
    value = e.interpret("[1, 2, 3]")
    assert value == [1, 2, 3]
