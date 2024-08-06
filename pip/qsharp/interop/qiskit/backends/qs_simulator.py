# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from collections import Counter
from concurrent.futures import Executor
import logging
from typing import Any, Dict, List, Optional
from uuid import uuid4

from qiskit import QuantumCircuit
from qiskit.providers import Options
from qiskit.transpiler.target import Target
from .... import Result, TargetProfile
from ..execution import SynchronousExecutor
from ..jobs import QsSimJob
from .qsbackend import QsBackend

logger = logging.getLogger(__name__)


def _map_qsharp_value_to_bit(v) -> str:
    if isinstance(v, Result):
        if v == Result.One:
            return "1"
        else:
            return "0"
    return str(v)


# Convert Q# output to the result format expected by Qiskit
def _to_qiskit_bitstring(obj):
    if isinstance(obj, tuple):
        return " ".join([_to_qiskit_bitstring(term) for term in obj])
    elif isinstance(obj, list):
        return "".join([_map_qsharp_value_to_bit(bit) for bit in obj])
    else:
        return obj


class QSharpSimulator(QsBackend):
    """
    A virtual backend for running Qiskit circuits using the Q# simulator.
    """

    # This init is included for the docstring
    # pylint: disable=useless-parent-delegation
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
            **options: Additional options for the execution.
              - name (str): The name of the circuit. This is used as the entry point for the program.
                  The circuit name will be used if not specified.
              - target_profile (TargetProfile): The target profile to use for the compilation.
              - shots (int): The number of shots to run the program for. Defaults to 1024.
              - seed (int): The seed to use for the random number generator. Defaults to None.
              - search_path (str): The path to search for imports. Defaults to '.'.
              - output_fn (Callable[[Output], None]): A callback function to
                  receive the output of the circuit. Defaults to None.
              - executor(ThreadPoolExecutor or other Executor):
                  The executor to be used to submit the job. Defaults to SynchronousExecutor.
        """

        super().__init__(target, transpile_options, skip_transpilation, **fields)

    @classmethod
    def _default_options(cls):
        return Options(
            name="program",
            search_path=".",
            shots=1024,
            seed=None,
            output_fn=None,
            target_profile=TargetProfile.Unrestricted,
            executor=SynchronousExecutor(),
        )

    def run(
        self,
        run_input: QuantumCircuit,
        **options,
    ) -> QsSimJob:
        """
        Runs the given QuantumCircuit using the Q# simulator.

        Args:
            run_input (QuantumCircuit): The QuantumCircuit to be executed.
            **options: Additional options for the execution.
              - name (str): The name of the circuit. This is used as the entry point for the program.
                  The circuit name will be used if not specified.
              - target_profile (TargetProfile): The target profile to use for the compilation.
              - shots (int): The number of shots to run the program for. Defaults to 1024.
              - seed (int): The seed to use for the random number generator. Defaults to None.
              - search_path (str): The path to search for imports. Defaults to '.'.
              - output_fn (Callable[[Output], None]): A callback function to
                  receive the output of the circuit.
              - executor(ThreadPoolExecutor or other Executor):
                  The executor to be used to submit the job.
        Returns:
            QSharpJob: The simulation job

        :raises QSharpError: If there is an error evaluating the source code.
        :raises QasmError: If there is an error parsing the source code.
        :raises AssertionError: If the run_input is not a QuantumCircuit.
        """

        if "name" not in options:
            options["name"] = run_input.name
        return self._run(run_input, **options)

    def _execute(self, programs: List[str], **input_params) -> Dict[str, Any]:
        exec_results = [_run_qasm3(qasm, **input_params) for qasm in programs]
        job_results = []

        shots = input_params.get("shots")
        if shots is None:
            raise ValueError("The number of shots must be specified.")

        for exec_result in exec_results:
            results = [_to_qiskit_bitstring(result) for result in exec_result]

            counts = Counter(results)
            counts_dict = dict(counts)
            probabilities = {
                bitstring: (count / shots) for bitstring, count in counts_dict.items()
            }

            job_result = {
                "data": {"counts": counts_dict, "probabilities": probabilities},
                "success": True,
                "header": {},
                "shots": shots,
            }
            job_results.append(job_result)

        # All of theses fields are required by the Result object
        result_dict = {
            "results": job_results,
            "qobj_id": str(uuid4()),
            "success": True,
        }

        return result_dict

    def _create_results(self, output: Dict[str, Any]) -> Any:
        from qiskit.result import Result

        result = Result.from_dict(output)
        return result

    def _submit_job(self, run_input: QuantumCircuit, **input_params) -> QsSimJob:
        job_id = str(uuid4())
        executor: Executor = input_params.pop("executor", SynchronousExecutor())
        job = QsSimJob(self, job_id, self.run_job, run_input, input_params, executor)
        job.submit()
        return job


def _run_qasm3(
    qasm: str,
    **options,
) -> Any:
    """
    Runs the supplied OpenQASM 3 program.
    Gates defined by stdgates.inc will be overridden with definitions
    from the Q# compiler.

    Any gates, such as matrix unitaries, that are not able to be
    transpiled will result in an error.

    Parameters:
    source (str): The input OpenQASM 3 string to be processed.
        **options: Additional keyword arguments to pass to the execution.
        - target_profile (TargetProfile): The target profile to use for execution.
        - name (str): The name of the circuit. This is used as the entry point for the program. Defaults to 'program'.
        - search_path (str): The optional search path for resolving qasm3 imports.
        - shots (int): The number of shots to run the program for. Defaults to 1.
        - seed (int): The seed to use for the random number generator.
        - output_fn (Optional[Callable[[Output], None]]): A callback function that will be called with each output. Defaults to None.

    :returns values: A result or runtime errors.

    :raises QSharpError: If there is an error evaluating the source code.
    :raises QasmError: If there is an error parsing the source code.
    """

    from ...._native import Output
    from ...._native import run_qasm3
    from ...._fs import read_file, list_directory, resolve
    from ...._http import fetch_github

    def callback(output: Output) -> None:
        print(output)

    output_fn = options.pop("output_fn", callback)

    return run_qasm3(
        qasm,
        output_fn,
        read_file,
        list_directory,
        resolve,
        fetch_github,
        **options,
    )
