import subprocess
from pathlib import Path
from wasmtime import Store, Module, Linker, FuncType, ValType
from disco.noise_model import create_default_noise_model
from disco.simulator import QirSim


def run_wasm(
    wasm_module: bytes, shots: int = 1, entry_point: str = "ENTRYPOINT__main"
) -> list[str]:
    simulator = QirSim()
    store = Store()
    module = Module(store.engine, wasm_module)

    results: list[str] = []
    all_results: list[list[str]] = []

    # Used by Grover's:
    #   void __quantum__qis__h__body(%Qubit*)
    #   void __quantum__qis__x__body(%Qubit*)
    #   void __quantum__qis__cz__body(%Qubit*, %Qubit*)
    #   void __quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
    #   void __quantum__qis__mresetz__body(%Qubit*, %Result*)
    # Required generally:
    #   i1 __quantum__qis__read_result__body(%Result*)
    #   void __quantum__rt__array_record_output(i64 size, i8* label_str)
    #   void __quantum__rt__result_record_output(%Result*, i8* label_str)
    # TODO: Add a (parameterized) Rz and an Sx (and a mov)

    # Special case mresetz to apply an Mz and then reset the qubit
    def qis_mresetz(qid: int, rid: int):
        simulator.apply_instrument("mz", [qid], [rid])
        simulator.apply_operation("reset", [qid])

    def rt_array_record_output(size: int, label_ptr: int):
        pass  # TODO

    def rt_result_record_output(rid: int, label_ptr: int):
        results.append(simulator.results[rid])

    linker = Linker(store.engine)
    linker.define_func(
        "env",
        "__quantum__qis__h__body",
        FuncType([ValType.i32()], []),
        lambda q: simulator.apply_operation("h", [q]),
    )
    linker.define_func(
        "env",
        "__quantum__qis__x__body",
        FuncType([ValType.i32()], []),
        lambda q: simulator.apply_operation("x", [q]),
    )
    linker.define_func(
        "env",
        "__quantum__qis__cz__body",
        FuncType([ValType.i32(), ValType.i32()], []),
        lambda c, t: simulator.apply_operation("cz", [c, t]),
    )
    linker.define_func(
        "env",
        "__quantum__qis__ccx__body",
        FuncType([ValType.i32(), ValType.i32(), ValType.i32()], []),
        lambda c1, c2, t: simulator.apply_operation("ccx", [c1, c2, t]),
    )

    linker.define_func(
        "env",
        "__quantum__qis__mresetz__body",
        FuncType([ValType.i32(), ValType.i32()], []),
        qis_mresetz,
    )

    linker.define_func(
        "env",
        "__quantum__qis__read_result__body",
        FuncType([ValType.i32()], [ValType.i32()]),
        lambda rid: int(simulator.results[rid]),
    )

    linker.define_func(
        "env",
        "__quantum__rt__array_record_output",
        FuncType([ValType.i64(), ValType.i32()], []),
        rt_array_record_output,
    )

    linker.define_func(
        "env",
        "__quantum__rt__result_record_output",
        FuncType([ValType.i32(), ValType.i32()], []),
        rt_result_record_output,
    )

    instance = linker.instantiate(store, module)

    entry_point_fn = instance.exports(store)[entry_point]

    for _ in range(shots):
        entry_point_fn(store)
        all_results.append(results)
        simulator.reset_simulator()
        results = []

    return all_results


def qir_to_wasm(qir_file_path: Path):
    # Get the directory containing this python script
    script_dir = qir_file_path.parent
    wasm_file_path = script_dir / "qir.wasm"

    # TODO: Configurable?
    clang_path = "/Users/billti/llvm18/bin/clang"

    subprocess.run(
        [
            clang_path,
            "-Oz",
            "--target=wasm32",
            "-nostdlib",
            "-Wno-override-module",
            "-Wl,--entry=ENTRYPOINT__main",
            "-Wl,--allow-undefined",
            "-o",
            str(wasm_file_path),
            str(qir_file_path),
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    return wasm_file_path
