# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest
import qsharp
import qsharp.code
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


def test_captured_stdout() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    f = io.StringIO()
    with redirect_stdout(f):
        result = qsharp.eval(
            '{Message("Hello, world!"); Message("Goodbye!")}', save_events=True
        )
    assert f.getvalue() == ""
    assert len(result["messages"]) == 2
    assert result["messages"][0] == "Hello, world!"
    assert result["messages"][1] == "Goodbye!"


def test_captured_matrix() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    f = io.StringIO()
    with redirect_stdout(f):
        result = qsharp.eval(
            "Std.Diagnostics.DumpOperation(1, qs => H(qs[0]))",
            save_events=True,
        )
    assert f.getvalue() == ""
    assert len(result["matrices"]) == 1
    assert (
        str(result["matrices"][0])
        == "MATRIX:\n 0.7071+0.0000𝑖 0.7071+0.0000𝑖\n 0.7071+0.0000𝑖 −0.7071+0.0000𝑖"
    )


def test_quantum_seed() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_quantum_seed(42)
    value1 = qsharp.eval(
        "{ use qs = Qubit[32]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }"
    )
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_quantum_seed(42)
    value2 = qsharp.eval(
        "{ use qs = Qubit[32]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }"
    )
    assert value1 == value2
    qsharp.set_quantum_seed(None)
    value3 = qsharp.eval(
        "{ use qs = Qubit[32]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }"
    )
    assert value1 != value3


def test_classical_seed() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_classical_seed(42)
    value1 = qsharp.eval(
        "{ mutable res = []; for _ in 0..15{ set res += [(Microsoft.Quantum.Random.DrawRandomInt(0, 100), Microsoft.Quantum.Random.DrawRandomDouble(0.0, 1.0))]; }; res }"
    )
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_classical_seed(42)
    value2 = qsharp.eval(
        "{ mutable res = []; for _ in 0..15{ set res += [(Microsoft.Quantum.Random.DrawRandomInt(0, 100), Microsoft.Quantum.Random.DrawRandomDouble(0.0, 1.0))]; }; res }"
    )
    assert value1 == value2
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.set_classical_seed(None)
    value3 = qsharp.eval(
        "{ mutable res = []; for _ in 0..15{ set res += [(Microsoft.Quantum.Random.DrawRandomInt(0, 100), Microsoft.Quantum.Random.DrawRandomDouble(0.0, 1.0))]; }; res }"
    )
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
    assert state_dump[2] == complex(1.0, 0.0)
    assert state_dump.as_dense_state() == [0, 0, 1, 0]
    qsharp.eval("X(q2);")
    state_dump = qsharp.dump_machine()
    assert state_dump.qubit_count == 2
    assert len(state_dump) == 1
    assert state_dump[3] == complex(1.0, 0.0)
    assert state_dump.as_dense_state() == [0, 0, 0, 1]
    qsharp.eval("H(q1);")
    state_dump = qsharp.dump_machine()
    assert state_dump.qubit_count == 2
    assert len(state_dump) == 2
    # Check that the state dump correctly supports iteration and membership checks
    for idx in state_dump:
        assert idx in state_dump
    # Check that the state dump is correct and equivalence check ignores global phase, allowing passing
    # in of different, potentially unnormalized states. The state should be
    # |01⟩: 0.7071+0.0000𝑖, |11⟩: −0.7071+0.0000𝑖
    assert state_dump.check_eq({1: complex(0.7071, 0.0), 3: complex(-0.7071, 0.0)})
    assert state_dump.as_dense_state() == [
        0,
        0.7071067811865476,
        0,
        -0.7071067811865476,
    ]
    assert state_dump.check_eq({1: complex(0.0, 0.7071), 3: complex(0.0, -0.7071)})
    assert state_dump.check_eq({1: complex(0.5, 0.0), 3: complex(-0.5, 0.0)})
    assert state_dump.check_eq(
        {1: complex(0.7071, 0.0), 3: complex(-0.7071, 0.0), 0: complex(0.0, 0.0)}
    )
    assert state_dump.check_eq([0.0, 0.5, 0.0, -0.5])
    assert state_dump.check_eq([0.0, 0.5001, 0.00001, -0.5], tolerance=1e-3)
    assert state_dump.check_eq(
        [complex(0.0, 0.0), complex(0.0, -0.5), complex(0.0, 0.0), complex(0.0, 0.5)]
    )
    assert not state_dump.check_eq({1: complex(0.7071, 0.0), 3: complex(0.7071, 0.0)})
    assert not state_dump.check_eq({1: complex(0.5, 0.0), 3: complex(0.0, 0.5)})
    assert not state_dump.check_eq({2: complex(0.5, 0.0), 3: complex(-0.5, 0.0)})
    assert not state_dump.check_eq([0.0, 0.5001, 0.0, -0.5], tolerance=1e-6)
    # Reset the qubits and apply a small rotation to q1, to confirm that tolerance applies to the dump
    # itself and not just the state.
    qsharp.eval("ResetAll([q1, q2]);")
    qsharp.eval("Ry(0.0001, q1);")
    state_dump = qsharp.dump_machine()
    assert state_dump.qubit_count == 2
    assert len(state_dump) == 2
    assert not state_dump.check_eq([1.0])
    assert state_dump.check_eq([0.99999999875, 0.0, 4.999999997916667e-05])
    assert state_dump.check_eq([1.0], tolerance=1e-4)


def test_dump_operation() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    res = qsharp.utils.dump_operation("qs => ()", 1)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0)],
    ]
    res = qsharp.utils.dump_operation("qs => H(qs[0])", 1)
    assert res == [
        [complex(0.707107, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(-0.707107, 0.0)],
    ]
    res = qsharp.utils.dump_operation("qs => CNOT(qs[0], qs[1])", 2)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
    ]
    res = qsharp.utils.dump_operation("qs => CCNOT(qs[0], qs[1], qs[2])", 3)
    assert res == [
        [
            complex(1.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(1.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(1.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(1.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(1.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(1.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(1.0, 0.0),
        ],
        [
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(0.0, 0.0),
            complex(1.0, 0.0),
            complex(0.0, 0.0),
        ],
    ]
    qsharp.eval(
        "operation ApplySWAP(qs : Qubit[]) : Unit is Ctl + Adj { SWAP(qs[0], qs[1]); }"
    )
    res = qsharp.utils.dump_operation("ApplySWAP", 2)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
    ]
    res = qsharp.utils.dump_operation("qs => ()", 8)
    for i in range(8):
        for j in range(8):
            if i == j:
                assert res[i][j] == complex(1.0, 0.0)
            else:
                assert res[i][j] == complex(0.0, 0.0)


def test_run_with_noise_produces_noisy_results() -> None:
    qsharp.init()
    qsharp.set_quantum_seed(0)
    result = qsharp.run(
        "{ mutable errors=0; for _ in 0..100 { use q1=Qubit(); use q2=Qubit(); H(q1); CNOT(q1, q2); if MResetZ(q1) != MResetZ(q2) { set errors+=1; } } errors }",
        shots=1,
        noise=qsharp.BitFlipNoise(0.1),
    )
    assert result[0] > 5
    result = qsharp.run(
        "{ mutable errors=0; for _ in 0..100 { use q=Qubit(); if MResetZ(q) != Zero { set errors+=1; } } errors }",
        shots=1,
        noise=qsharp.BitFlipNoise(0.1),
    )
    assert result[0] > 5


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


def test_init_from_provider_name() -> None:
    config = qsharp.init(target_name="ionq.simulator")
    assert config._config["targetProfile"] == "base"
    config = qsharp.init(target_name="rigetti.sim.qvm")
    assert config._config["targetProfile"] == "base"
    config = qsharp.init(target_name="quantinuum.sim")
    assert config._config["targetProfile"] == "adaptive_ri"
    config = qsharp.init(target_name="Quantinuum")
    assert config._config["targetProfile"] == "adaptive_ri"
    config = qsharp.init(target_name="IonQ")
    assert config._config["targetProfile"] == "base"


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
        == "[{'result': Zero, 'events': [Hello, world!], 'messages': ['Hello, world!'], 'matrices': [], 'dumps': []}, {'result': Zero, 'events': [Hello, world!], 'messages': ['Hello, world!'], 'matrices': [], 'dumps': []}, {'result': Zero, 'events': [Hello, world!], 'messages': ['Hello, world!'], 'matrices': [], 'dumps': []}]"
    )
    stdout = capsys.readouterr().out
    assert stdout == ""
    assert called


def test_run_with_invalid_shots_produces_error() -> None:
    qsharp.init()
    qsharp.eval('operation Foo() : Result { Message("Hello, world!"); Zero }')
    try:
        qsharp.run("Foo()", -1)
    except qsharp.QSharpError as e:
        assert str(e) == "The number of shots must be greater than 0."
    else:
        assert False

    try:
        qsharp.run("Foo()", 0)
    except qsharp.QSharpError as e:
        assert str(e) == "The number of shots must be greater than 0."
    else:
        assert False


def test_target_profile_str_values_match_enum_values() -> None:
    target_profile = qsharp.TargetProfile.Base
    str_value = str(target_profile)
    assert str_value == "Base"
    target_profile = qsharp.TargetProfile.Adaptive_RI
    str_value = str(target_profile)
    assert str_value == "Adaptive_RI"
    target_profile = qsharp.TargetProfile.Unrestricted
    str_value = str(target_profile)
    assert str_value == "Unrestricted"


def test_target_profile_from_str_match_enum_values() -> None:
    target_profile = qsharp.TargetProfile.Base
    str_value = str(target_profile)
    assert qsharp.TargetProfile.from_str(str_value) == target_profile
    target_profile = qsharp.TargetProfile.Adaptive_RI
    str_value = str(target_profile)
    assert qsharp.TargetProfile.from_str(str_value) == target_profile
    target_profile = qsharp.TargetProfile.Unrestricted
    str_value = str(target_profile)
    assert qsharp.TargetProfile.from_str(str_value) == target_profile
    with pytest.raises(ValueError):
        qsharp.TargetProfile.from_str("Invalid")


def test_callables_exposed_into_env() -> None:
    qsharp.init()
    qsharp.eval("function Four() : Int { 4 }")
    assert qsharp.code.Four() == 4, "callable should be available"
    qsharp.eval("function Add(a : Int, b : Int) : Int { a + b }")
    assert qsharp.code.Four() == 4, "first callable should still be available"
    assert qsharp.code.Add(2, 3) == 5, "second callable should be available"
    # After init, the callables should be cleared and no longer available
    qsharp.init()
    with pytest.raises(AttributeError):
        qsharp.code.Four()


def test_callable_exposed_into_env_complex_types() -> None:
    qsharp.eval(
        "function Complicated(a : Int, b : (Double, BigInt)) : ((Double, BigInt), Int) { (b, a) }"
    )
    assert qsharp.code.Complicated(2, (3.0, 4000000000000000000000)) == (
        (3.0, 4000000000000000000000),
        2,
    ), "callables that take complex types should marshall them correctly"


def test_callable_exposed_into_env_with_array() -> None:
    qsharp.init()
    qsharp.eval("function Smallest(a : Int[]) : Int { Std.Math.Min(a)}")
    assert (
        qsharp.code.Smallest([1, 2, 3, 0, 4, 5]) == 0
    ), "callable that takes array should work"


def test_callable_with_int_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : Int) : Int { a }")
    assert qsharp.code.Identity(4) == 4
    with pytest.raises(TypeError):
        qsharp.code.Identity("4")
    with pytest.raises(TypeError):
        qsharp.code.Identity(4.0)
    with pytest.raises(OverflowError):
        qsharp.code.Identity(4000000000000000000000)
    with pytest.raises(TypeError):
        qsharp.code.Identity([4])


def test_callable_with_double_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : Double) : Double { a }")
    assert qsharp.code.Identity(4.0) == 4.0
    assert qsharp.code.Identity(4) == 4.0
    with pytest.raises(TypeError):
        qsharp.code.Identity("4")
    with pytest.raises(TypeError):
        qsharp.code.Identity([4])


def test_callable_with_bigint_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : BigInt) : BigInt { a }")
    assert qsharp.code.Identity(4000000000000000000000) == 4000000000000000000000
    with pytest.raises(TypeError):
        qsharp.code.Identity("4")
    with pytest.raises(TypeError):
        qsharp.code.Identity(4.0)


def test_callable_with_string_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : String) : String { a }")
    assert qsharp.code.Identity("4") == "4"
    with pytest.raises(TypeError):
        qsharp.code.Identity(4)
    with pytest.raises(TypeError):
        qsharp.code.Identity(4.0)
    with pytest.raises(TypeError):
        qsharp.code.Identity([4])


def test_callable_with_bool_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : Bool) : Bool { a }")
    assert qsharp.code.Identity(True) == True
    with pytest.raises(TypeError):
        qsharp.code.Identity("4")
    with pytest.raises(TypeError):
        qsharp.code.Identity(4)
    with pytest.raises(TypeError):
        qsharp.code.Identity(4.0)
    with pytest.raises(TypeError):
        qsharp.code.Identity([4])


def test_callable_with_array_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : Int[]) : Int[] { a }")
    assert qsharp.code.Identity([4, 5, 6]) == [4, 5, 6]
    assert qsharp.code.Identity([]) == []
    assert qsharp.code.Identity((4, 5, 6)) == [4, 5, 6]
    with pytest.raises(TypeError):
        qsharp.code.Identity(4)
    with pytest.raises(TypeError):
        qsharp.code.Identity("4")
    with pytest.raises(TypeError):
        qsharp.code.Identity(4.0)
    with pytest.raises(TypeError):
        qsharp.code.Identity([1, 2, 3.0])


def test_callable_with_tuple_exposed_into_env_fails_incorrect_types() -> None:
    qsharp.init()
    qsharp.eval("function Identity(a : (Int, Double)) : (Int, Double) { a }")
    assert qsharp.code.Identity((4, 5.0)) == (4, 5.0)
    assert qsharp.code.Identity((4, 5)) == (4, 5.0)
    assert qsharp.code.Identity([4, 5.0]) == (4, 5.0)
    with pytest.raises(qsharp.QSharpError):
        qsharp.code.Identity((4, 5, 6))
    with pytest.raises(TypeError):
        qsharp.code.Identity(4)
    with pytest.raises(TypeError):
        qsharp.code.Identity("4")
    with pytest.raises(TypeError):
        qsharp.code.Identity(4.0)
    with pytest.raises(TypeError):
        qsharp.code.Identity([4.0, 5])


def test_callables_in_namespaces_exposed_into_env_submodules_and_removed_on_reinit() -> (
    None
):
    qsharp.init()
    # callables should be created with their namespaces
    qsharp.eval("namespace Test { function Four() : Int { 4 } }")
    qsharp.eval("function Identity(a : Int) : Int { a }")
    # should be able to import callables from env and namespace submodule
    from qsharp.code import Identity
    from qsharp.code.Test import Four

    assert Identity(4) == 4
    assert Four() == 4
    qsharp.init()
    # namespaces should be removed
    with pytest.raises(AttributeError):
        qsharp.code.Test
    with pytest.raises(AttributeError):
        qsharp.code.Identity()
    # imported callables should fail gracefully
    with pytest.raises(qsharp.QSharpError):
        Four()


def test_callables_with_unsupported_types_raise_errors_on_call() -> None:
    qsharp.init()
    qsharp.eval("function Unsupported(a : Int, q : Qubit) : Unit { }")
    with pytest.raises(qsharp.QSharpError, match="unsupported input type: `Qubit`"):
        qsharp.code.Unsupported()


def test_callables_with_unsupported_types_in_tuples_raise_errors_on_call() -> None:
    qsharp.init()
    qsharp.eval("function Unsupported(q : (Int, Qubit)[]) : Unit { }")
    with pytest.raises(qsharp.QSharpError, match="unsupported input type: `Qubit`"):
        qsharp.code.Unsupported()


def test_callables_with_unsupported_return_types_raise_errors_on_call() -> None:
    qsharp.init()
    qsharp.eval('function Unsupported() : Qubit { fail "won\'t be called" }')
    with pytest.raises(qsharp.QSharpError, match="unsupported output type: `Qubit`"):
        qsharp.code.Unsupported()


def test_callables_with_unsupported_udt_types_raise_errors_on_call() -> None:
    qsharp.init()
    qsharp.eval("function Unsupported(a : Std.Math.Complex) : Unit { }")
    with pytest.raises(
        qsharp.QSharpError, match='unsupported input type: `UDT<"Complex":'
    ):
        qsharp.code.Unsupported()


def test_callable_with_unsupported_udt_return_types_raise_errors_on_call() -> None:
    qsharp.init()
    qsharp.eval('function Unsupported() : Std.Math.Complex { fail "won\'t be called" }')
    with pytest.raises(
        qsharp.QSharpError, match='unsupported output type: `UDT<"Complex":'
    ):
        qsharp.code.Unsupported()


def test_struct_call_constructor_not_exposed_into_env() -> None:
    qsharp.init()
    qsharp.eval("struct CustomUDT { a : Int }")
    with pytest.raises(AttributeError):
        qsharp.code.CustomUDT


def test_lambdas_not_exposed_into_env() -> None:
    qsharp.init()
    qsharp.eval("a -> a + 1")
    assert not hasattr(qsharp.code, "<lambda>")
    qsharp.eval("q => I(q)")
    assert not hasattr(qsharp.code, "<lambda>")
