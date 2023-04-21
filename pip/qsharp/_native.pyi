# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Tuple
from typing import List
from typing import Literal
from enum import Enum

class Evaluator:
    """A Q# evaluator."""

    def __init__(self) -> None:
        """Initializes a new Q# evaluator."""
        ...
    def eval(self, expr: str) -> Tuple[str, str, List[ExecutionError]]:
        """Evaluates a Q# expression.
        returns: A tuple of the expression's result and simulation data.
        .0 is the result of the expression.
        .1 is the output from the simulation.
        .2 is the error output.
        """
        ...

class Result(Enum):
    """A measurement result."""
    Zero = 0
    One = 1

class Pauli(Enum):
    I = 0
    X = 1
    Y = 2
    Z = 3

class ExecutionError:
    error_type: Literal["CompilationError", "RuntimeError"]
    message: str