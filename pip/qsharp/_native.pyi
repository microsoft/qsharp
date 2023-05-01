# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.


from enum import Enum
from typing import Any, List, Literal, Tuple


class Interpreter:
    """A Q# interpreter."""

    def __init__(self) -> None:
        """Initializes a new Q# interpreter."""
        ...

    def interpret(self, expr: str) -> Tuple[Any,
                                            List[Output], List[Error]]:
        """ Interprets a line of Q#.

        :param expr: The line of Q# to interpret.

        :returns (value, outputs, errors):
            value: The value of the last statement in the line.
            outputs: A list of outputs from the line. An output can be a state or a message.
            errors: A list of errors from the line. Errors can be compilation or runtime errors.
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


class Error:
    """An error returned from the Q# interpreter."""
    error_type: Literal['CompilationError', 'RuntimeError']
    message: str
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
