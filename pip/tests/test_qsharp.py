# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import qsharp
import qsharp.utils
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

def test_quantum_seed() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_quantum_seed(42)
    value1 = qsharp.eval("{ use qs = Qubit[32]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }")
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_quantum_seed(42)
    value2 = qsharp.eval("{ use qs = Qubit[32]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }")
    assert value1 == value2
    qsharp.set_quantum_seed(None)
    value3 = qsharp.eval("{ use qs = Qubit[32]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }")
    assert value1 != value3


def test_classical_seed() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_classical_seed(42)
    value1 = qsharp.eval("{ mutable res = []; for _ in 0..15{ set res += [(Microsoft.Quantum.Random.DrawRandomInt(0, 100), Microsoft.Quantum.Random.DrawRandomDouble(0.0, 1.0))]; }; res }")
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_classical_seed(42)
    value2 = qsharp.eval("{ mutable res = []; for _ in 0..15{ set res += [(Microsoft.Quantum.Random.DrawRandomInt(0, 100), Microsoft.Quantum.Random.DrawRandomDouble(0.0, 1.0))]; }; res }")
    assert value1 == value2
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_classical_seed(None)
    value3 = qsharp.eval("{ mutable res = []; for _ in 0..15{ set res += [(Microsoft.Quantum.Random.DrawRandomInt(0, 100), Microsoft.Quantum.Random.DrawRandomDouble(0.0, 1.0))]; }; res }")
    assert value1 != value3


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
    assert state_dump[2] == (1.0, 0.0)
    qsharp.eval("X(q2);")
    state_dump = qsharp.dump_machine()
    assert state_dump.qubit_count == 2
    assert len(state_dump) == 1
    assert state_dump[3] == (1.0, 0.0)

def test_dump_operation() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    res = qsharp.utils.dump_operation("qs => ()", 1)
    assert res == [[complex(1.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(1.0, 0.0)]]
    res = qsharp.utils.dump_operation("qs => H(qs[0])", 1)
    assert res == [[complex(0.707107, 0.0), complex(0.707107, 0.0)],
                   [complex(0.707107, 0.0), complex(-0.707107, 0.0)]]
    res = qsharp.utils.dump_operation("qs => CNOT(qs[0], qs[1])", 2)
    assert res == [[complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)]]
    res = qsharp.utils.dump_operation("qs => CCNOT(qs[0], qs[1], qs[2])", 3)
    assert res == [[complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)]]
    qsharp.eval("operation ApplySWAP(qs : Qubit[]) : Unit { SWAP(qs[0], qs[1]); }")
    res = qsharp.utils.dump_operation("ApplySWAP", 2)
    assert res == [[complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
                   [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)]]
    res = qsharp.utils.dump_operation("qs => ()", 8)
    for i in range(8):
        for j in range(8):
            if i == j:
                assert res[i][j] == complex(1.0, 0.0)
            else:
                assert res[i][j] == complex(0.0, 0.0)


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
