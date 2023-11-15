# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import qsharp
from contextlib import redirect_stdout
import io

# Tests for the Python library for Q#


def test_stdout() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Full)
    f = io.StringIO()
    with redirect_stdout(f):
        result = qsharp.eval('Message("Hello, world!")')

    assert result is None
    assert f.getvalue() == "Hello, world!\n"


def test_stdout_multiple_lines() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Full)
    f = io.StringIO()
    with redirect_stdout(f):
        qsharp.eval(
            """
        use q = Qubit();
        Microsoft.Quantum.Diagnostics.DumpMachine();
        Message("Hello!");
        """
        )

    assert f.getvalue() == "STATE:\n|0⟩: 1.0000+0.0000𝑖\nHello!\n"

def test_dump_machine() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Full)
    qsharp.eval(
        """
    use q1 = Qubit();
    use q2 = Qubit();
    X(q1);
    """
    )
    state_dump = qsharp.dump_machine()
    assert state_dump.qubit_count == 2
    assert len(state_dump) == 1
    assert state_dump[1] == (1.0, 0.0)
    qsharp.eval("X(q2);")
    state_dump = qsharp.dump_machine()
    assert state_dump.qubit_count == 2
    assert len(state_dump) == 1
    assert state_dump[3] == (1.0, 0.0)


def test_compile_qir_input_data() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Base)
    qsharp.eval("operation Program() : Result { use q = Qubit(); return M(q) }")
    operation = qsharp.compile("Program()")
    qir = operation._repr_qir_()
    assert isinstance(qir, bytes)
