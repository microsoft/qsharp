# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
from typing import Any, Callable, Optional, Dict, List, Tuple

# pylint: disable=unused-argument
# E302 is fighting with the formatter for number of blank lines
# flake8: noqa: E302

class OutputSemantics(Enum):
    """
    Represents the output semantics for OpenQASM 3 compilation.
    Each has implications on the output of the compilation
    and the semantic checks that are performed.
    """

    Qiskit: OutputSemantics
    """
    The output is in Qiskit format meaning that the output
    is all of the classical registers, in reverse order
    in which they were added to the circuit with each
    bit within each register in reverse order.
    """

    OpenQasm: OutputSemantics
    """
    [OpenQASM 3 has two output modes](https://openqasm.com/language/directives.html#input-output)
    - If the programmer provides one or more `output` declarations, then
        variables described as outputs will be returned as output.
        The spec make no mention of endianness or order of the output.
    - Otherwise, assume all of the declared variables are returned as output.
    """

    ResourceEstimation: OutputSemantics
    """
    No output semantics are applied. The entry point returns `Unit`.
    """

class ProgramType(Enum):
    """
    Represents the type of compilation output to create
    """

    File: ProgramType
    """
    Creates an operation in a namespace as if the program is a standalone
    file. Inputs are lifted to the operation params. Output are lifted to
    the operation return type. The operation is marked as `@EntryPoint`
    as long as there are no input parameters.
    """

    Operation: ProgramType
    """
    Programs are compiled to a standalone function. Inputs are lifted to
    the operation params. Output are lifted to the operation return type.
    """

    Fragments: ProgramType
    """
    Creates a list of statements from the program. This is useful for
    interactive environments where the program is a list of statements
    imported into the current scope.
    This is also useful for testing individual statements compilation.
    """

class TargetProfile(Enum):
    """
    A Q# target profile.

    A target profile describes the capabilities of the hardware or simulator
    which will be used to run the Q# program.
    """

    @classmethod
    def from_str(cls, value: str) -> TargetProfile: ...
    """
    Creates a target profile from a string.
    :param value: The string to parse.
    :raises ValueError: If the string does not match any target profile.
    """

    Base: TargetProfile
    """
    Target supports the minimal set of capabilities required to run a quantum
    program.

    This option maps to the Base Profile as defined by the QIR specification.
    """

    Adaptive_RI: TargetProfile
    """
    Target supports the Adaptive profile with integer computation and qubit
    reset capabilities.

    This profile includes all of the required Adaptive Profile
    capabilities, as well as the optional integer computation and qubit
    reset capabilities, as defined by the QIR specification.
    """

    Unrestricted: TargetProfile
    """
    Describes the unrestricted set of capabilities required to run any Q# program.
    """

class Interpreter:
    """A Q# interpreter."""

    def __init__(
        self,
        target_profile: TargetProfile,
        language_features: Optional[List[str]],
        project_root: Optional[str],
        read_file: Callable[[str], Tuple[str, str]],
        list_directory: Callable[[str], List[Dict[str, str]]],
        resolve_path: Callable[[str, str], str],
    ) -> None:
        """
        Initializes the Q# interpreter.

        :param target_profile: The target profile to use for the interpreter.
        :param project_root: A directory that contains a `qsharp.json` manifest.
        :param read_file: A function that reads a file from the file system.
        :param list_directory: A function that lists the contents of a directory.
        :param resolve_path: A function that joins path segments and normalizes the resulting path.
        """
        ...

    def interpret(self, input: str, output_fn: Callable[[Output], None]) -> Any:
        """
        Interprets Q# source code.

        :param input: The Q# source code to interpret.
        :param output_fn: A callback function that will be called with each output.

        :returns value: The value returned by the last statement in the input.

        :raises QSharpError: If there is an error interpreting the input.
        """
        ...

    def run(
        self,
        entry_expr: str,
        output_fn: Callable[[Output], None],
        noise: Optional[Tuple[float, float, float]],
    ) -> Any:
        """
        Runs the given Q# expression with an independent instance of the simulator.

        :param entry_expr: The entry expression.
        :param output_fn: A callback function that will be called with each output.
        :param noise: A tuple with probabilities of Pauli-X, Pauli-Y, and Pauli-Z errors
            to use in simulation as a parametric Pauli noise.

        :returns values: A result or runtime errors.

        :raises QSharpError: If there is an error interpreting the input.
        """
        ...

    def qir(self, entry_expr: str) -> str:
        """
        Generates QIR from Q# source code.

        :param entry_expr: The entry expression.

        :returns qir: The QIR string.
        """
        ...

    def circuit(
        self,
        entry_expr: Optional[str],
        operation: Optional[str],
    ) -> Circuit:
        """
        Synthesizes a circuit for a Q# program. Either an entry
        expression or an operation must be provided.

        :param entry_expr: An entry expression.

        :param operation: The operation to synthesize. This can be a name of
        an operation of a lambda expression. The operation must take only
        qubits or arrays of qubits as parameters.

        :raises QSharpError: If there is an error synthesizing the circuit.
        """
        ...

    def estimate(self, entry_expr: str, params: str) -> str:
        """
        Estimates resources for Q# source code.

        :param entry_expr: The entry expression.
        :param params: The parameters to configure estimation.

        :returns resources: The estimated resources.
        """
        ...

    def set_quantum_seed(self, seed: Optional[int]) -> None:
        """
        Sets the seed for the quantum random number generator.

        :param seed: The seed to use for the quantum random number generator. If None,
            the seed will be generated from entropy.
        """
        ...

    def set_classical_seed(self, seed: Optional[int]) -> None:
        """
        Sets the seed for the classical random number generator.

        :param seed: The seed to use for the classical random number generator. If None,
            the seed will be generated from entropy.
        """
        ...

    def dump_machine(self) -> StateDumpData:
        """
        Returns the sparse state vector of the simulator as a StateDump object.

        :returns: The state of the simulator.
        """
        ...

    def dump_circuit(self) -> Circuit:
        """
        Dumps the current circuit state of the interpreter.

        This circuit will contain the gates that have been applied
        in the simulator up to the current point.
        """
        ...

class Result(Enum):
    """
    A Q# measurement result.
    """

    Zero: int
    One: int

class Pauli(Enum):
    """
    A Q# Pauli operator.
    """

    I: int
    X: int
    Y: int
    Z: int

class Output:
    """
    An output returned from the Q# interpreter.
    Outputs can be a state dumps or messages. These are normally printed to the console.
    """

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def _repr_markdown_(self) -> Optional[str]: ...
    def state_dump(self) -> Optional[StateDumpData]: ...

class StateDumpData:
    """
    A state dump returned from the Q# interpreter.
    """

    """
    The number of allocated qubits at the time of the dump.
    """
    qubit_count: int

    """
    Get the amplitudes of the state vector as a dictionary from state integer to
    complex amplitudes.
    """
    def get_dict(self) -> dict: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def _repr_markdown_(self) -> str: ...
    def _repr_latex_(self) -> Optional[str]: ...

class Circuit:
    def json(self) -> str: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class QSharpError(BaseException):
    """
    An error returned from the Q# interpreter.
    """

    ...

class QasmError(BaseException):
    """
    An error returned from the OpenQASM parser.
    """

    ...

def physical_estimates(logical_resources: str, params: str) -> str:
    """
    Estimates physical resources from pre-calculated logical resources.

    :param logical_resources: The logical resources to estimate from.
    :param params: The parameters to configure physical estimation.

    :returns resources: The estimated resources.
    """
    ...

def resource_estimate_qasm3(
    source: str,
    job_params: str,
    read_file: Callable[[str], Tuple[str, str]],
    list_directory: Callable[[str], List[Dict[str, str]]],
    resolve_path: Callable[[str, str], str],
    fetch_github: Callable[[str, str, str, str], str],
    **kwargs
) -> str:
    """
    Estimates the resource requirements for executing QASM3 source code.

    Note:
        This call while exported is not intended to be used directly by the user.
        It is intended to be used by the Python wrapper which will handle the
        callbacks and other Python specific details.

    Args:
        source (str): The QASM3 source code to estimate the resource requirements for.
        job_params (str): The parameters for the job.
        read_file (Callable[[str], Tuple[str, str]]): A callable that reads a file and returns its content and path.
        list_directory (Callable[[str], List[Dict[str, str]]]): A callable that lists the contents of a directory.
        resolve_path (Callable[[str, str], str]): A callable that resolves a file path given a base path and a relative path.
        fetch_github (Callable[[str, str, str, str], str]): A callable that fetches a file from GitHub.
        **kwargs: Additional keyword arguments to pass to the execution.
          - name (str): The name of the circuit. This is used as the entry point for the program. Defaults to 'program'.
          - search_path (str): The optional search path for resolving imports.
    Returns:
        str: The estimated resource requirements for executing the QASM3 source code.
    """
    ...

def run_qasm3(
    source: str,
    output_fn: Callable[[Output], None],
    read_file: Callable[[str], Tuple[str, str]],
    list_directory: Callable[[str], List[Dict[str, str]]],
    resolve_path: Callable[[str, str], str],
    fetch_github: Callable[[str, str, str, str], str],
    **kwargs
) -> Any:
    """
    Executes QASM3 source code using the specified target profile.

    Note:
        This call while exported is not intended to be used directly by the user.
        It is intended to be used by the Python wrapper which will handle the
        callbacks and other Python specific details.

    Args:
        source (str): The QASM3 source code to execute.
        output_fn (Callable[[Output], None]): The function to handle the output of the execution.
        read_file (Callable[[str], Tuple[str, str]]): The function to read a file and return its contents.
        list_directory (Callable[[str], List[Dict[str, str]]]): The function to list the contents of a directory.
        resolve_path (Callable[[str, str], str]): The function to resolve a path given a base path and a relative path.
        fetch_github (Callable[[str, str, str, str], str]): The function to fetch a file from GitHub.
        **kwargs: Additional keyword arguments to pass to the execution.
          - target_profile (TargetProfile): The target profile to use for execution.
          - name (str): The name of the circuit. This is used as the entry point for the program. Defaults to 'program'.
          - search_path (str): The optional search path for resolving imports.
          - shots (int): The number of shots to run the program for. Defaults to 1.
          - seed (int): The seed to use for the random number generator.

    Returns:
        Any: The result of the execution.
    """
    ...

def compile_qasm3_to_qir(
    source: str,
    read_file: Callable[[str], Tuple[str, str]],
    list_directory: Callable[[str], List[Dict[str, str]]],
    resolve_path: Callable[[str, str], str],
    fetch_github: Callable[[str, str, str, str], str],
    **kwargs
) -> str:
    """
    Converts a Qiskit QuantumCircuit to QIR (Quantum Intermediate Representation).

    Note:
        This call while exported is not intended to be used directly by the user.
        It is intended to be used by the Python wrapper which will handle the
        callbacks and other Python specific details.

    Args:
        source (str): The QASM3 source code to estimate the resource requirements for.
        read_file (Callable[[str], Tuple[str, str]]): A callable that reads a file and returns its content and path.
        list_directory (Callable[[str], List[Dict[str, str]]]): A callable that lists the contents of a directory.
        resolve_path (Callable[[str, str], str]): A callable that resolves a file path given a base path and a relative path.
        fetch_github (Callable[[str, str, str, str], str]): A callable that fetches a file from GitHub.
        **kwargs: Additional keyword arguments to pass to the execution.
          - name (str): The name of the circuit. This is used as the entry point for the program.
          - entry_expr (str, optional): The entry expression for the QIR conversion. Defaults to None.
          - target_profile (TargetProfile): The target profile to use for code generation.
          - search_path (Optional[str]): The optional search path for resolving file references.

    Returns:
        str: The converted QIR code as a string.
    """
    ...

def compile_qasm3_to_qsharp(
    source: str,
    read_file: Callable[[str], Tuple[str, str]],
    list_directory: Callable[[str], List[Dict[str, str]]],
    resolve_path: Callable[[str, str], str],
    fetch_github: Callable[[str, str, str, str], str],
    **kwargs
) -> str:
    """
    Converts a Qiskit QuantumCircuit to Q#.

    Note:
        This call while exported is not intended to be used directly by the user.
        It is intended to be used by the Python wrapper which will handle the
        callbacks and other Python specific details.

    Args:
        source (str): The QASM3 source code to estimate the resource requirements for.
        read_file (Callable[[str], Tuple[str, str]]): A callable that reads a file and returns its content and path.
        list_directory (Callable[[str], List[Dict[str, str]]]): A callable that lists the contents of a directory.
        resolve_path (Callable[[str, str], str]): A callable that resolves a file path given a base path and a relative path.
        fetch_github (Callable[[str, str, str, str], str]): A callable that fetches a file from GitHub.
        **kwargs: Additional keyword arguments to pass to the execution.
          - name (str): The name of the circuit. This is used as the entry point for the program.
          - search_path (Optional[str]): The optional search path for resolving file references.

    Returns:
        str: The converted Q# code as a string.
    """
    ...
