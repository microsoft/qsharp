# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
from typing import Any, Callable

class Interpreter:
    """A Q# interpreter."""

    def __init__(self) -> None:
        """Initializes a new Q# interpreter."""
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
