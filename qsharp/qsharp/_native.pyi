# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Tuple

class Evaluator:
    """A Q# evaluator."""

    def __init__(self) -> None:
        """Initializes a new Q# evaluator."""
        ...
    def eval(self, expr: str) -> Tuple[str, str, str]:
        """Evaluates a Q# expression.
        returns: A tuple of the expression's result and simulation data.
        .0 is the result of the expression.
        .1 is the output from the simulation.
        .2 is the error output.
        """
        ...
