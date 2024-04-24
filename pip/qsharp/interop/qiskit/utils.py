# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.


from concurrent.futures import Executor, Future
from typing import Callable, Optional, Any
from ..._native import compile_qasm3_to_qir
from ..._fs import read_file, list_directory, resolve
from ..._http import fetch_github
from ... import TargetProfile

from qiskit import QuantumCircuit, transpile
from qiskit.dagcircuit import DAGCircuit
from qiskit.providers import BackendV2
from qiskit.qasm3.exporter import Exporter
from qiskit.transpiler import PassManager
from qiskit.transpiler.basepasses import TransformationPass
from qiskit.transpiler.passes import RemoveBarriers
from qiskit.transpiler.passes.utils import control_flow


class SynchronousExecutor(Executor):
    def submit(self, fn: Callable[..., Any], /, *args, **kwargs) -> Future:
        future: Future = Future()
        try:
            result = fn(*args, **kwargs)
            future.set_result(result)
        except Exception as e:
            future.set_exception(e)
        return future

    def shutdown(self, wait=True, *, cancel_futures=False) -> None:
        # No resources to clean up in this simple synchronous executor
        pass


class RemoveRemoveDelays(TransformationPass):
    """Return a circuit with any delay removed.

    This transformation is not semantics preserving.
    """

    @control_flow.trivial_recurse
    def run(self, dag: DAGCircuit) -> DAGCircuit:
        """Run the RemoveRemoveDelays pass on `dag`."""

        dag.remove_all_ops_named("delay")

        return dag


def _transpile(
    circuit: QuantumCircuit, backend: BackendV2, **options
) -> QuantumCircuit:
    remove_barriers = not options.pop("supports_barrier", False)
    remove_delays = not options.pop("supports_delay", False)
    pass_manager = PassManager()
    if remove_barriers:
        pass_manager.append(RemoveBarriers())
    if remove_delays:
        pass_manager.append(RemoveRemoveDelays())
    circuit = pass_manager.run(circuit)

    return transpile(
        circuit,
        backend=backend,
        target=backend.target,
        optimization_level=0,
    )


def _convert_qiskit_to_qasm3(
    circuit: QuantumCircuit, backend: BackendV2, **options
) -> str:
    transpiled_circuit = _transpile(circuit, backend, **options)
    # Disable aliasing until we decide want to support it
    # The exporter defaults to only having the U gate.
    # When it sees the stdgates.inc in the default includes list, it adds
    # bodyless symbols for that fixed gate set.
    # We set the basis gates for any gates that we want that wouldn't
    # be defined when stdgates.inc is included.
    exporter = Exporter(
        alias_classical_registers=False,
        allow_aliasing=False,
        basis_gates=["rxx", "ryy", "rzz"],
    )
    qasm3_source = exporter.dumps(transpiled_circuit)
    return qasm3_source


def _convert_qasm3_to_qir(
    source: str,
    name: str,
    target_profile: TargetProfile = TargetProfile.Base,
    entry_expr: Optional[str] = None,
    search_path: Optional[str] = None,
) -> str:
    return compile_qasm3_to_qir(
        source,
        name,
        target_profile,
        entry_expr,
        search_path,
        read_file,
        list_directory,
        resolve,
        fetch_github,
    )
