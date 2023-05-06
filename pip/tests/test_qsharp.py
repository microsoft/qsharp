# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import qsharp
from contextlib import redirect_stdout
import io
import pytest


# Tests for the Python library for Q#


def test_stdout() -> None:
    f = io.StringIO()
    with redirect_stdout(f):
        result = qsharp.interpret('Message("Hello, world!")')

    assert result is None
    assert f.getvalue() == "Hello, world!\n"


def test_stdout_multiple_lines() -> None:
    f = io.StringIO()
    with redirect_stdout(f):
        qsharp.interpret("""
        use q = Qubit();
        Microsoft.Quantum.Diagnostics.DumpMachine();
        Message("Hello!");
        """)

    assert f.getvalue() == "STATE:\n|0⟩: 1.0000+0.0000i\nHello!\n"


def test_compilation_error() -> None:
    with pytest.raises(qsharp.CompilationException) as excinfo:
        qsharp.interpret("operation Foo() {}")
    assert excinfo.value.diagnostics.__repr__(
    ) == "[CompilationError: syntax error]"


def test_runtime_error() -> None:
    with pytest.raises(qsharp.RuntimeException) as excinfo:
        qsharp.interpret("""
    operation ReleaseOneQubit(): Unit {
        use q = Qubit();
        X(q);
    }
    ReleaseOneQubit();
    """)
    assert excinfo.value.diagnostics.__repr__(
    ) == "[RuntimeError: Qubit1 released while not in |0⟩ state]"
