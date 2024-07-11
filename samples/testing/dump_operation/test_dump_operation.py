import qsharp
from qsharp.utils import dump_operation

def test_empty_operation() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    res = dump_operation("qs => ()", 1)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0)],
    ]


def test_single_qubit_not_gate() -> None:
    res = dump_operation("qs => X(qs[0])", 1)
    assert res == [
        [complex(0.0, 0.0), complex(1.0, 0.0)],
        [complex(1.0, 0.0), complex(0.0, 0.0)],
    ]

def test_single_qubit_superposition() -> None:
    res = dump_operation("qs => H(qs[0])", 1)
    assert res == [
        [complex(0.707107, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(-0.707107, 0.0)],
    ]

def test_two_qubit_cnot_gate() -> None:
    res = dump_operation("qs => CNOT(qs[0], qs[1])", 2)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
    ]

def test_custom_operation() -> None:
    qsharp.eval(
        "operation ApplySWAP(qs : Qubit[]) : Unit is Ctl + Adj { SWAP(qs[0], qs[1]); }"
    )
    res = dump_operation("ApplySWAP", 2)

    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
    ]

def test_operation_no_args_in_qsharp_file() -> None:
    qsharp.init(project_root='.')
    res = dump_operation("SWAP.ApplySWAP", 2)

    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
    ]

def test_operation_with_args_in_qsharp_file() -> None:
    qsharp.init(project_root='.')

    res0 = dump_operation("BellState.AllBellStates(_, 0)", 2)

    assert res0 == [
        [complex(0.707107, 0.0), complex(0.0,0.0), complex(0.707107, 0.0), complex(0.0,0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0)],
    ]

    res1 = dump_operation("BellState.AllBellStates(_, 1)", 2)

    assert res1 == [
        [complex(0.707107, 0.0), complex(0.0,0.0), complex(0.707107, 0.0), complex(0.0,0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
    ]

    res2 = dump_operation("BellState.AllBellStates(_, 2)", 2)

    assert res2 == [
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0)],
    ],  f"got {res2}"

    res3 = dump_operation("BellState.AllBellStates(_, 3)", 2)

    assert res3 == [
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
        [complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
    ]
