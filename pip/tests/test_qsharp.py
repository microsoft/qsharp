# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import qsharp
from contextlib import redirect_stdout
import io

# Tests for the Python library for Q#


def test_stdout() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    f = io.StringIO()
    with redirect_stdout(f):
        result = qsharp.eval('Message("Hello, world!")')

    assert result is None
    assert f.getvalue() == "Hello, world!\n"


def test_stdout_multiple_lines() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
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
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
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


def test_compile_qir_str() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Base)
    qsharp.eval("operation Program() : Result { use q = Qubit(); return M(q) }")
    operation = qsharp.compile("Program()")
    qir = str(operation)
    assert "define void @ENTRYPOINT__main()" in qir


def test_run_with_result(capsys) -> None:
    qsharp.init()
    qsharp.eval('operation Foo() : Result { Message("Hello, world!"); Zero }')
    results = qsharp.run("Foo()", 3)
    assert results == [qsharp.Result.Zero, qsharp.Result.Zero, qsharp.Result.Zero]
    stdout = capsys.readouterr().out
    assert stdout == "Hello, world!\nHello, world!\nHello, world!\n"


def test_run_with_result_callback(capsys) -> None:
    def on_result(result):
        nonlocal called
        called = True
        assert result["result"] == qsharp.Result.Zero
        assert str(result["events"]) == "[Hello, world!]"

    called = False
    qsharp.init()
    qsharp.eval('operation Foo() : Result { Message("Hello, world!"); Zero }')
    results = qsharp.run("Foo()", 3, on_result=on_result, save_events=True)
    assert (
        str(results)
        == "[{'result': Zero, 'events': [Hello, world!]}, {'result': Zero, 'events': [Hello, world!]}, {'result': Zero, 'events': [Hello, world!]}]"
    )
    stdout = capsys.readouterr().out
    assert stdout == ""
    assert called
