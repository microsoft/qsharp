# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from abc import ABC, abstractmethod
import datetime
import time
import logging
from typing import Dict, Any, List

from qiskit.providers import BackendV2
from qiskit.circuit import (
    Measure,
    Parameter,
    QuantumCircuit,
    Reset,
    Store,
)
from qiskit.circuit.controlflow import (
    IfElseOp,
    ForLoopOp,
    WhileLoopOp,
    SwitchCaseOp,
    ControlFlowOp,
    BreakLoopOp,
    ContinueLoopOp,
)
from qiskit.circuit import Barrier, Delay
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
from qiskit.transpiler.target import Target
from qiskit.result import Result
from qsharp import QSharpError

from ..utils import _convert_qiskit_to_qasm3

from .... import TargetProfile
from ..jobs import QsJob

logger = logging.getLogger(__name__)


class QsBackend(BackendV2, ABC):
    """
    A virtual backend for transpiling to a Q# ecosystem compatible format.
    """

    def __init__(
        self,
        target=None,
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
        # we need to set the target after the options are set
        # so that the target_profile can be used to determine
        # which gates/instructions are available
        if target is not None:
            # update the properties so that we are internally consistent
            self._options["supports_barrier"] = target.instruction_supported("barrier")
            self._options["supports_delay"] = target.instruction_supported("delay")
            self._target = target
        else:
            self._target = self._create_target()

    def _create_target(self) -> Target:
        target = Target(num_qubits=2 ^ 64 - 1)
        if self._options["target_profile"] != TargetProfile.Base:
            target.add_instruction(BreakLoopOp, name="break")
            target.add_instruction(ContinueLoopOp, name="continue")
            target.add_instruction(ControlFlowOp, name="control_flow")
            target.add_instruction(IfElseOp, name="if_else")
            target.add_instruction(SwitchCaseOp, name="switch_case")
            target.add_instruction(WhileLoopOp, name="while_loop")

        target.add_instruction(Store, name="store")

        if self._options["supports_barrier"]:
            target.add_instruction(Barrier, name="barrier")
        if self._options["supports_delay"]:
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
        qasm = [
            _convert_qiskit_to_qasm3(circuit, self, **options) for circuit in run_input
        ]
        return qasm

    @abstractmethod
    def _create_results(self, output: Dict[str, Any]) -> Any:
        pass
