# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
from typing import Any, Callable, ClassVar

class TargetProfile:
    """
    A Q# target profile.

    The target is the hardware or simulator which will be used to run the Q# program.
    The target profile is a description of a target's capabilities.
    """

    Full: ClassVar[Any]
    """
    Describes the full set of capabilities required to run any Q# program.

    This option maps to the Full Profile as defined by the QIR specification.
    """

    Base: ClassVar[Any]
    """
    Target supports the minimal set of capabilities required to run a quantum
    program.

    This option maps to the Base Profile as defined by the QIR specification.
    """

class Interpreter:
    """A Q# interpreter."""

    def __init__(self, target_profile: TargetProfile) -> None:
        """
        Initializes the Q# interpreter.

        :param target_profile: The target profile to use for the interpreter.
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
        self, entry_expr: str, shots: int, output_fn: Callable[[Output], None]
    ) -> Any:
        """
        Runs the given Q# expressin for the given number of shots.
        Each shot uses an independent instance of the simulator.

        :param entry_expr: The entry expression.
        :param shots: The number of shots to run.
        :param output_fn: A callback function that will be called with each output.

        :returns values: A list of results or runtime errors.

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

class QSharpError(BaseException):
    """
    An error returned from the Q# interpreter.
    """

    ...
