# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from abc import ABC, abstractmethod
import datetime
import logging
import time
from typing import Dict, Any, List, Optional

from qiskit import transpile
from qiskit.circuit import (
    Barrier,
    Delay,
    Measure,
    Parameter,
    QuantumCircuit,
    Reset,
    Store,
)
from qiskit.circuit.controlflow import (
    BreakLoopOp,
    ContinueLoopOp,
    ControlFlowOp,
    ForLoopOp,
    IfElseOp,
    SwitchCaseOp,
    WhileLoopOp,
)
from qiskit.circuit.library.standard_gates import (
    CHGate,
    CCXGate,
    CXGate,
    CYGate,
    CZGate,
    CRXGate,
    CRYGate,
    CRZGate,
    RXGate,
    RXXGate,
    RYGate,
    RYYGate,
    RZGate,
    RZZGate,
    HGate,
    SGate,
    SdgGate,
    SwapGate,
    TGate,
    TdgGate,
    XGate,
    YGate,
    ZGate,
    IGate,
)
from qiskit.qasm3.exporter import Exporter
from qiskit.providers import BackendV2, Options
from qiskit.result import Result
from qiskit.transpiler import PassManager
from qiskit.transpiler.passes import RemoveBarriers
from qiskit.transpiler.target import Target

from ..jobs import QsJob
from ..passes import RemoveRemoveDelays
from .... import QSharpError, TargetProfile


logger = logging.getLogger(__name__)


def filter_kwargs(func, kwargs):
    import inspect

    sig = inspect.signature(func)
    supported_args = set(sig.parameters.keys())
    extracted_kwargs = {
        k: kwargs.pop(k) for k in list(kwargs.keys()) if k in supported_args
    }
    return extracted_kwargs


class QsBackend(BackendV2, ABC):
    """
    A virtual backend for transpiling to a Q# ecosystem compatible format.
    """

    def __init__(
        self,
        target: Optional[Target] = None,
        transpile_options: Optional[Dict[str, Any]] = None,
        skip_transpilation: bool = False,
        **fields,
    ):
        """
        Parameters:
            target (Target): The target to use for the backend.
            **options: Additional keyword arguments to pass to the
                execution used by subclasses.
        """
        super().__init__(
            name="QSharpSimulator",
            description="A virtual BackendV2 for transpiling to a Q# compatible format.",
            backend_version="0.0.1",
        )

        if fields is not None:
            self.set_options(**fields)

        self._transpile_options = Options(supports_barrier=False, supports_delay=False)
        self._skip_transpilation = skip_transpilation

        # we need to set the target after the options are set
        # so that the target_profile can be used to determine
        # which gates/instructions are available
        if target is not None:
            # update the properties so that we are internally consistent
            self._transpile_options.update_options(
                **{
                    "supports_barrier": target.instruction_supported("barrier"),
                    "supports_delay": target.instruction_supported("delay"),
                }
            )

            self._target = target
        else:
            self._target = self._create_target()

        if transpile_options is not None:
            self._transpile_options.update_options(**transpile_options)

    def _create_target(self) -> Target:
        num_qubits = None
        target = Target(num_qubits=num_qubits)
        if self._options["target_profile"] != TargetProfile.Base:
            target.add_instruction(BreakLoopOp, name="break")
            target.add_instruction(ContinueLoopOp, name="continue")
            target.add_instruction(ControlFlowOp, name="control_flow")
            target.add_instruction(IfElseOp, name="if_else")
            target.add_instruction(SwitchCaseOp, name="switch_case")
            target.add_instruction(WhileLoopOp, name="while_loop")

        target.add_instruction(Store, name="store")

        if self._transpile_options["supports_barrier"]:
            target.add_instruction(Barrier, name="barrier")
        if self._transpile_options["supports_delay"]:
            target.add_instruction(Delay, name="delay")

        # For loops should be fully deterministic in Qiskit/QASM
        target.add_instruction(ForLoopOp, name="for_loop")
        target.add_instruction(Measure, name="measure")

        # While reset is technically not supported in base profile,
        # the compiler can use decompositions to implement workarounds
        target.add_instruction(Reset, name="reset")

        target.add_instruction(CCXGate, name="ccx")
        target.add_instruction(CXGate, name="cx")
        target.add_instruction(CYGate, name="cy")
        target.add_instruction(CZGate, name="cz")

        target.add_instruction(RXGate(Parameter("theta")), name="rx")
        target.add_instruction(RXXGate(Parameter("theta")), name="rxx")
        target.add_instruction(CRXGate(Parameter("theta")), name="crx")

        target.add_instruction(RYGate(Parameter("theta")), name="ry")
        target.add_instruction(RYYGate(Parameter("theta")), name="ryy")
        target.add_instruction(CRYGate(Parameter("theta")), name="cry")

        target.add_instruction(RZGate(Parameter("theta")), name="rz")
        target.add_instruction(RZZGate(Parameter("theta")), name="rzz")
        target.add_instruction(CRZGate(Parameter("theta")), name="crz")

        target.add_instruction(HGate, name="h")

        target.add_instruction(SGate, name="s")
        target.add_instruction(SdgGate, name="sdg")

        target.add_instruction(SwapGate, name="swap")

        target.add_instruction(TGate, name="t")
        target.add_instruction(TdgGate, name="tdg")

        target.add_instruction(XGate, name="x")
        target.add_instruction(YGate, name="y")
        target.add_instruction(ZGate, name="z")

        target.add_instruction(IGate, name="id")

        target.add_instruction(CHGate, name="ch")

        return target

    @property
    def target(self) -> Target:
        """Returns the target of the Backend object."""
        return self._target

    @property
    def max_circuits(self):
        """
        Returns the maximum number of circuits that can be executed simultaneously.
        """
        return None

    @abstractmethod
    def _execute(self, programs: List[str], **input_params) -> Dict[str, Any]:
        """Execute circuits on the backend.

        Parameters:
            circuits (List of str): simulator qasm input.
            input_params (Dict): configuration for simulation/compilation.

        Returns:
            dict: return a dictionary of results.
        """

    @abstractmethod
    def run(
        self,
        run_input: QuantumCircuit,
        **options,
    ) -> QsJob:
        pass

    def _run(
        self,
        run_input: QuantumCircuit,
        **options,
    ) -> QsJob:
        assert isinstance(run_input, QuantumCircuit)

        # Get out default options
        # Look at all of the kwargs and see if they match any of the options
        # If they do, set the option to the value of the kwarg as an override
        # We only to remove the options that are in the backend options for
        # the run so that other options can be passed to other calls.
        input_params: Dict[str, Any] = vars(self.options).copy()
        for opt in options.copy():
            if opt in input_params:
                input_params[opt] = options.pop(opt)

        return self._submit_job([run_input], **input_params)

    def run_job(
        self, run_input: List[QuantumCircuit], job_id: str, **input_params
    ) -> Result:
        start = time.time()

        programs = self._compile(run_input, **input_params)

        output = self._execute(programs, **input_params)

        if not isinstance(output, dict):
            logger.error("%s: run failed.", self.name)
            if output:
                logger.error("Output: %s", output)
            raise QSharpError("Run terminated without valid output.")

        output["job_id"] = job_id
        output["date"] = datetime.datetime.now().isoformat()
        output["backend_name"] = self.name
        output["backend_version"] = self.backend_version
        output["time_taken"] = time.time() - start

        return self._create_results(output)

    @abstractmethod
    def _submit_job(self, run_input: QuantumCircuit, **input_params) -> QsJob:
        pass

    def _compile(self, run_input: List[QuantumCircuit], **options) -> List[str]:
        # for each run input, convert to qasm3
        qasm = [self.qasm3(circuit, **options) for circuit in run_input]
        return qasm

    @abstractmethod
    def _create_results(self, output: Dict[str, Any]) -> Any:
        pass

    def _transpile(self, circuit: QuantumCircuit, **options) -> QuantumCircuit:
        if self._skip_transpilation:
            return circuit

        remove_barriers = not options.pop("supports_barrier", False)
        remove_delays = not options.pop("supports_delay", False)
        pass_manager = PassManager()
        if remove_barriers:
            pass_manager.append(RemoveBarriers())
        if remove_delays:
            pass_manager.append(RemoveRemoveDelays())
        circuit = pass_manager.run(circuit)

        if "optimization_level" not in options:
            options["optimization_level"] = 0

        backend = options.pop("backend", self)
        target = options.pop("target", self.target)
        remove_final_measurements = options.pop("remove_final_measurements", False)

        orig = self.target.num_qubits
        try:
            self.target.num_qubits = circuit.num_qubits
            transpiled_circuit = transpile(
                circuit, backend=backend, target=target, **options
            )

            if remove_final_measurements:
                transpiled_circuit.remove_final_measurements(inplace=True)

            return transpiled_circuit
        finally:
            self.target.num_qubits = orig

    def _build_transpile_options(self, **kwargs) -> Dict[str, Any]:
        params: Dict[str, Any] = vars(self._transpile_options).copy()
        for opt in kwargs.copy():
            params[opt] = kwargs.pop(opt)
        return params

    def _build_qasm_export_options(self, kwargs) -> Dict[str, Any]:
        # Disable aliasing until we decide want to support it
        # The exporter defaults to only having the U gate.
        # When it sees the stdgates.inc in the default includes list, it adds
        # bodyless symbols for that fixed gate set.
        # We set the basis gates for any gates that we want that wouldn't
        # be defined when stdgates.inc is included.

        includes = kwargs.pop("includes", ("stdgates.inc",))
        alias_classical_registers = kwargs.pop("alias_classical_registers", False)
        allow_aliasing = kwargs.pop("allow_aliasing", False)
        disable_constants = kwargs.pop("disable_constants", True)
        basis_gates = kwargs.pop("basis_gates", ["rxx", "ryy", "rzz"])

        return {
            "includes": includes,
            "alias_classical_registers": alias_classical_registers,
            "allow_aliasing": allow_aliasing,
            "disable_constants": disable_constants,
            "basis_gates": basis_gates,
        }

    def transpile(self, circuit: QuantumCircuit, **options) -> QuantumCircuit:
        transpile_options = filter_kwargs(transpile, options)
        transpile_options = self._build_transpile_options(**transpile_options)
        transpiled_circuit = self._transpile(circuit, **transpile_options)
        return transpiled_circuit

    def qasm3(self, circuit: QuantumCircuit, **options) -> str:
        export_options = self._build_qasm_export_options(options)
        transpiled_circuit = self.transpile(circuit, **options)

        exporter = Exporter(**export_options)
        qasm3_source = exporter.dumps(transpiled_circuit)
        return qasm3_source

    def qir(
        self,
        circuit: QuantumCircuit,
        target_profile: Optional[TargetProfile] = None,
        entry_expr: Optional[str] = None,
        search_path: Optional[str] = None,
        **kwargs,
    ) -> str:
        """
        Converts a Qiskit QuantumCircuit to QIR (Quantum Intermediate Representation).

        Args:
            circuit ('QuantumCircuit'): The input Qiskit QuantumCircuit object.
            target_profile (TargetProfile, optional): The target profile for the backend. Defaults to backend config value.
            entry_expr (str, optional): The entry expression for the QIR conversion. Defaults to None.
            search_path (str, optional): The search path for the backend. Defaults to '.'.

        Returns:
            str: The converted QIR code as a string.
        """
        target_profile = target_profile or self.options.target_profile

        qasm3_source = self.qasm3(circuit, **kwargs)
        return self._qir(
            qasm3_source, circuit.name, target_profile, entry_expr, search_path
        )

    def _qir(
        self,
        source: str,
        name: str,
        target_profile: TargetProfile = TargetProfile.Base,
        entry_expr: Optional[str] = None,
        search_path: Optional[str] = None,
    ) -> str:
        from ...._native import compile_qasm3_to_qir
        from ...._fs import read_file, list_directory, resolve
        from ...._http import fetch_github

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
