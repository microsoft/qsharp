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

def test_swap_operation_in_qsharp_file() -> None:
    qsharp.init(project_root='.')
    res = dump_operation("SWAP.ApplySWAP", 2)

    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
    ]

def test_bell_state_operation_in_qsharp_file() -> None:
    qsharp.init(project_root='.')
    res = dump_operation("BellState.PrepareBellState", 2)

    assert res == [
        [(0.707107+0j), 0j, (0.707107+0j), 0j],
        [0j, (0.707107+0j), 0j, (0.707107+0j)],
        [0j, (0.707107+0j), 0j, (-0.707107-0j)],
        [(0.707107+0j), 0j, (-0.707107-0j), 0j],
    ], f"Got {res}"
