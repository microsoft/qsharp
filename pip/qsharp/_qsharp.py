# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import Interpreter, TargetProfile, StateDump, QSharpError, Output
from typing import Any, Callable, Dict, Optional, TypedDict, Union, List
from .estimator._estimator import EstimatorResult, EstimatorParams
import json

_interpreter = None


class Config:
    _config: Dict[str, str]
    """
    Configuration hints for the language service.
    """

    def __init__(self, target_profile: TargetProfile):
        if target_profile == TargetProfile.Unrestricted:
            self._config = {"targetProfile": "unrestricted"}
        elif target_profile == TargetProfile.Base:
            self._config = {"targetProfile": "base"}

    def __repr__(self) -> str:
        return "Q# initialized with configuration: " + str(self._config)

    # See https://ipython.readthedocs.io/en/stable/config/integrating.html#rich-display
    # See https://ipython.org/ipython-doc/3/notebook/nbformat.html#display-data
    # This returns a custom MIME-type representation of the Q# configuration.
    # This data will be available in the cell output, but will not be displayed
    # to the user, as frontends would not know how to render the custom MIME type.
    # Editor services that interact with the notebook frontend
    # (i.e. the language service) can read and interpret the data.
    def _repr_mimebundle_(
        self, include: Union[Any, None] = None, exclude: Union[Any, None] = None
    ) -> Dict[str, Dict[str, str]]:
        return {"application/x.qsharp-config": self._config}


def init(
    *,
    target_profile: TargetProfile = TargetProfile.Unrestricted,
    project_root: Optional[str] = None,
) -> Config:
    """
    Initializes the Q# interpreter.

    :param target_profile: Setting the target profile allows the Q#
        interpreter to generate programs that are compatible
        with a specific target. See :py:class: `qsharp.TargetProfile`.

    :param project_root: An optional path to a root directory with a Q# project to include.
        It must contain a qsharp.json project manifest.
    """
    from ._fs import read_file, list_directory, exists, join

    global _interpreter

    manifest_descriptor = None
    if project_root is not None:
        qsharp_json = join(project_root, "qsharp.json")
        if not exists(qsharp_json):
            raise QSharpError(
                f"{qsharp_json} not found. qsharp.json should exist at the project root and be a valid JSON file."
            )

        manifest_descriptor = {}
        manifest_descriptor["manifest_dir"] = project_root

        try:
            (_, file_contents) = read_file(qsharp_json)
        except Exception as e:
            raise QSharpError(
                f"Error reading {qsharp_json}. qsharp.json should exist at the project root and be a valid JSON file."
            ) from e

        try:
            manifest_descriptor["manifest"] = json.loads(file_contents)
        except Exception as e:
            raise QSharpError(
                f"Error parsing {qsharp_json}. qsharp.json should exist at the project root and be a valid JSON file."
            ) from e

    _interpreter = Interpreter(
        target_profile, manifest_descriptor, read_file, list_directory
    )

    # Return the configuration information to provide a hint to the
    # language service through the cell output.
    return Config(target_profile)


def get_interpreter() -> Interpreter:
    """
    Returns the Q# interpreter.

    :returns: The Q# interpreter.
    """
    global _interpreter
    if _interpreter is None:
        init()
        assert _interpreter is not None, "Failed to initialize the Q# interpreter."
    return _interpreter


def eval(source: str) -> Any:
    """
    Evaluates Q# source code.

    Output is printed to console.

    :param source: The Q# source code to evaluate.
    :returns value: The value returned by the last statement in the source code.
    :raises QSharpError: If there is an error evaluating the source code.
    """

    def callback(output: Output) -> None:
        print(output)

    return get_interpreter().interpret(source, callback)


class ShotResult(TypedDict):
    """
    A single result of a shot.
    """

    events: List[Output]
    result: Any


def run(
    entry_expr: str,
    shots: int,
    *,
    on_result: Optional[Callable[[ShotResult], None]] = None,
    save_events: bool = False,
) -> List[Any]:
    """
    Runs the given Q# expression for the given number of shots.
    Each shot uses an independent instance of the simulator.

    :param entry_expr: The entry expression.
    :param shots: The number of shots to run.
    :param on_result: A callback function that will be called with each result.
    :param save_events: If true, the output of each shot will be saved. If false, they will be printed.

    :returns values: A list of results or runtime errors. If `save_events` is true,
    a List of ShotResults is returned.

    :raises QSharpError: If there is an error interpreting the input.
    """

    results: List[ShotResult] = []

    def print_output(output: Output) -> None:
        print(output)

    def on_save_events(output: Output) -> None:
        # Append the output to the last shot's output list
        results[-1]["events"].append(output)

    for _ in range(shots):
        results.append({"result": None, "events": []})
        run_results = get_interpreter().run(
            entry_expr, on_save_events if save_events else print_output
        )
        results[-1]["result"] = run_results
        if on_result:
            on_result(results[-1])

    if save_events:
        return results
    else:
        return [shot["result"] for shot in results]


# Class that wraps generated QIR, which can be used by
# azure-quantum as input data.
#
# This class must implement the QirRepresentable protocol
# that is defined by the azure-quantum package.
# See: https://github.com/microsoft/qdk-python/blob/fcd63c04aa871e49206703bbaa792329ffed13c4/azure-quantum/azure/quantum/target/target.py#L21
class QirInputData:
    # The name of this variable is defined
    # by the protocol and must remain unchanged.
    _name: str

    def __init__(self, name: str, ll_str: str):
        self._name = name
        self._ll_str = ll_str

    # The name of this method is defined
    # by the protocol and must remain unchanged.
    def _repr_qir_(self, **kwargs) -> bytes:
        return self._ll_str.encode("utf-8")

    def __str__(self) -> str:
        return self._ll_str


def compile(entry_expr: str) -> QirInputData:
    """
    Compiles the Q# source code into a program that can be submitted to a target.

    :param entry_expr: The Q# expression that will be used as the entrypoint
        for the program.

    :returns QirInputData: The compiled program.

    To get the QIR string from the compiled program, use `str()`.

    Example:

    .. code-block:: python
        program = qsharp.compile("...")
        with open('myfile.ll', 'w') as file:
            file.write(str(program))
    """
    ll_str = get_interpreter().qir(entry_expr)
    return QirInputData("main", ll_str)


def estimate(
    entry_expr, params: Optional[Union[Dict[str, Any], List, EstimatorParams]] = None
) -> EstimatorResult:
    """
    Estimates resources for Q# source code.

    :param entry_expr: The entry expression.
    :param params: The parameters to configure physical estimation.

    :returns resources: The estimated resources.
    """
    if params is None:
        params = [{}]
    elif isinstance(params, EstimatorParams):
        if params.has_items:
            params = params.as_dict()["items"]
        else:
            params = [params.as_dict()]
    elif isinstance(params, dict):
        params = [params]
    return EstimatorResult(
        json.loads(get_interpreter().estimate(entry_expr, json.dumps(params)))
    )

def set_quantum_seed(seed: Optional[int]) -> None:
    """
    Sets the seed for the random number generator used for quantum measurements.
    This applies to all Q# code executed, compiled, or estimated.

    :param seed: The seed to use for the quantum random number generator.
        If None, the seed will be generated from entropy.
    """
    get_interpreter().set_quantum_seed(seed)

def set_classical_seed(seed: Optional[int]) -> None:
    """
    Sets the seed for the random number generator used for standard
    library classical random number operations.
    This applies to all Q# code executed, compiled, or estimated.

    :param seed: The seed to use for the classical random number generator.
        If None, the seed will be generated from entropy.
    """
    get_interpreter().set_classical_seed(seed)

def dump_machine() -> StateDump:
    """
    Returns the sparse state vector of the simulator as a StateDump object.

    :returns: The state of the simulator.
    """
    return get_interpreter().dump_machine()
