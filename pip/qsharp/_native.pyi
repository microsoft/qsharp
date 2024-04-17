# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
from typing import Any, Callable, ClassVar, Optional, Dict, List

class TargetProfile:
    """
    A Q# target profile.

    A target profile describes the capabilities of the hardware or simulator
    which will be used to run the Q# program.
    """

    Base: ClassVar[Any]
    """
    Target supports the minimal set of capabilities required to run a quantum
    program.

    This option maps to the Base Profile as defined by the QIR specification.
    """

    Quantinuum: ClassVar[Any]
    """
    Target supports Quantinuum profile.

    This profile includes all of the required Adaptive Profile
    capabilities, as well as the optional integer computation and qubit
    reset capabilities, as defined by the QIR specification.
    """

    Unrestricted: ClassVar[Any]
    """
    Describes the unrestricted set of capabilities required to run any Q# program.
    """

class Interpreter:
    """A Q# interpreter."""

    def __init__(
        self,
        target_profile: TargetProfile,
        language_features: Optional[List[str]],
        manifest_descriptor: Optional[Dict[str, str]],
        read_file: Callable[[str], str],
        list_directory: Callable[[str], str],
    ) -> None:
        """
        Initializes the Q# interpreter.

        :param target_profile: The target profile to use for the interpreter.
        :param manifest_descriptor: A dictionary that represents the manifest descriptor
        :param read_file: A function that reads a file from the file system.
        :param list_directory: A function that lists the contents of a directory.
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

    def run(self, entry_expr: str, output_fn: Callable[[Output], None]) -> Any:
        """
        Runs the given Q# expression with an independent instance of the simulator.

        :param entry_expr: The entry expression.
        :param output_fn: A callback function that will be called with each output.

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
    def _repr_html_(self) -> str: ...
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
    def _repr_html_(self) -> str: ...

class Circuit:
    def json(self) -> str: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class QSharpError(BaseException):
    """
    An error returned from the Q# interpreter.
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
