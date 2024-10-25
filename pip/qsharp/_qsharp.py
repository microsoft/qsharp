# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._native import (
    Interpreter,
    TargetProfile,
    StateDumpData,
    QSharpError,
    Output,
    Circuit,
)
from typing import (
    Any,
    Callable,
    Dict,
    Optional,
    Tuple,
    TypedDict,
    Union,
    List,
    overload,
)
from .estimator._estimator import EstimatorResult, EstimatorParams
import json
import os

_interpreter = None

# Check if we are running in a Jupyter notebook to use the IPython display function
_in_jupyter = False
try:
    from IPython.display import display

    if get_ipython().__class__.__name__ == "ZMQInteractiveShell":  # type: ignore
        _in_jupyter = True  # Jupyter notebook or qtconsole
except:
    pass


# Reporting execution time during IPython cells requires that IPython
# gets pinged to ensure it understands the cell is active. This is done by
# requesting a display id without displaying any content, avoiding any UI changes
# that would be visible to the user.
def ipython_helper():
    try:
        if __IPYTHON__:  # type: ignore
            from IPython.display import display

            display(display_id=True)
    except NameError:
        pass


class Config:
    _config: Dict[str, str]
    """
    Configuration hints for the language service.
    """

    def __init__(
        self,
        target_profile: TargetProfile,
        language_features: Optional[List[str]],
        manifest: Optional[str],
        project_root: Optional[str],
    ):
        if target_profile == TargetProfile.Adaptive_RI:
            self._config = {"targetProfile": "adaptive_ri"}
        elif target_profile == TargetProfile.Base:
            self._config = {"targetProfile": "base"}
        elif target_profile == TargetProfile.Unrestricted:
            self._config = {"targetProfile": "unrestricted"}

        self._config["languageFeatures"] = language_features
        self._config["manifest"] = manifest
        if project_root:
            # For now, we only support local project roots, so use a file schema in the URI.
            # In the future, we may support other schemes, such as github, if/when
            # we have VS Code Web + Jupyter support.
            normalized_root = os.path.normpath(os.path.join(os.getcwd(), project_root))
            self._config["projectRoot"] = "file://" + normalized_root

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


class PauliNoise(Tuple[float, float, float]):
    """
    The Pauli noise to use in simulation represented
    as probabilities of Pauli-X, Pauli-Y, and Pauli-Z errors
    """

    def __new__(cls, x: float, y: float, z: float):
        if x < 0 or y < 0 or z < 0:
            raise ValueError("Pauli noise probabilities must be non-negative.")
        if x + y + z > 1:
            raise ValueError("The sum of Pauli noise probabilities must be at most 1.")
        return super().__new__(cls, (x, y, z))


class DepolarizingNoise(PauliNoise):
    """
    The depolarizing noise to use in simulation.
    """

    def __new__(cls, p: float):
        return super().__new__(cls, p / 3, p / 3, p / 3)


class BitFlipNoise(PauliNoise):
    """
    The bit flip noise to use in simulation.
    """

    def __new__(cls, p: float):
        return super().__new__(cls, p, 0, 0)


class PhaseFlipNoise(PauliNoise):
    """
    The phase flip noise to use in simulation.
    """

    def __new__(cls, p: float):
        return super().__new__(cls, 0, 0, p)


def init(
    *,
    target_profile: TargetProfile = TargetProfile.Unrestricted,
    target_name: Optional[str] = None,
    project_root: Optional[str] = None,
    language_features: Optional[List[str]] = None,
) -> Config:
    """
    Initializes the Q# interpreter.

    :param target_profile: Setting the target profile allows the Q#
        interpreter to generate programs that are compatible
        with a specific target. See :py:class: `qsharp.TargetProfile`.

    :param target_name: An optional name of the target machine to use for inferring the compatible
        target_profile setting.

    :param project_root: An optional path to a root directory with a Q# project to include.
        It must contain a qsharp.json project manifest.
    """
    from ._fs import read_file, list_directory, exists, join, resolve
    from ._http import fetch_github

    global _interpreter

    if isinstance(target_name, str):
        target = target_name.split(".")[0].lower()
        if target == "ionq" or target == "rigetti":
            target_profile = TargetProfile.Base
        elif target == "quantinuum":
            target_profile = TargetProfile.Adaptive_RI
        else:
            raise QSharpError(
                f'target_name "{target_name}" not recognized. Please set target_profile directly.'
            )

    manifest_contents = None
    if project_root is not None:
        qsharp_json = join(project_root, "qsharp.json")
        if not exists(qsharp_json):
            raise QSharpError(
                f"{qsharp_json} not found. qsharp.json should exist at the project root and be a valid JSON file."
            )

        try:
            (_, manifest_contents) = read_file(qsharp_json)
        except Exception as e:
            raise QSharpError(
                f"Error reading {qsharp_json}. qsharp.json should exist at the project root and be a valid JSON file."
            ) from e

    _interpreter = Interpreter(
        target_profile,
        language_features,
        project_root,
        read_file,
        list_directory,
        resolve,
        fetch_github,
    )

    # Return the configuration information to provide a hint to the
    # language service through the cell output.
    return Config(target_profile, language_features, manifest_contents, project_root)


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
    ipython_helper()

    def callback(output: Output) -> None:
        if _in_jupyter:
            try:
                display(output)
                return
            except:
                # If IPython is not available, fall back to printing the output
                pass
        print(output, flush=True)

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
    noise: Optional[
        Union[
            Tuple[float, float, float],
            PauliNoise,
            BitFlipNoise,
            PhaseFlipNoise,
            DepolarizingNoise,
        ]
    ] = None,
) -> List[Any]:
    """
    Runs the given Q# expression for the given number of shots.
    Each shot uses an independent instance of the simulator.

    :param entry_expr: The entry expression.
    :param shots: The number of shots to run.
    :param on_result: A callback function that will be called with each result.
    :param save_events: If true, the output of each shot will be saved. If false, they will be printed.
    :param noise: The noise to use in simulation.

    :returns values: A list of results or runtime errors. If `save_events` is true,
    a List of ShotResults is returned.

    :raises QSharpError: If there is an error interpreting the input.
    """
    ipython_helper()

    if shots < 1:
        raise QSharpError("The number of shots must be greater than 0.")

    results: List[ShotResult] = []

    def print_output(output: Output) -> None:
        if _in_jupyter:
            try:
                display(output)
                return
            except:
                # If IPython is not available, fall back to printing the output
                pass
        print(output, flush=True)

    def on_save_events(output: Output) -> None:
        # Append the output to the last shot's output list
        results[-1]["events"].append(output)

    for shot in range(shots):
        results.append({"result": None, "events": []})
        run_results = get_interpreter().run(
            entry_expr,
            on_save_events if save_events else print_output,
            noise,
        )
        results[-1]["result"] = run_results
        if on_result:
            on_result(results[-1])
        # For every shot after the first, treat the entry expression as None to trigger
        # a rerun of the last executed expression without paying the cost for any additional
        # compilation.
        entry_expr = None

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
    ipython_helper()

    ll_str = get_interpreter().qir(entry_expr)
    return QirInputData("main", ll_str)


def circuit(
    entry_expr: Optional[str] = None, *, operation: Optional[str] = None
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
    ipython_helper()
    return get_interpreter().circuit(entry_expr, operation)


def estimate(
    entry_expr: str,
    params: Optional[Union[Dict[str, Any], List, EstimatorParams]] = None,
) -> EstimatorResult:
    """
    Estimates resources for Q# source code.

    :param entry_expr: The entry expression.
    :param params: The parameters to configure physical estimation.

    :returns `EstimatorResult`: The estimated resources.
    """

    ipython_helper()

    def _coerce_estimator_params(
        params: Optional[Union[Dict[str, Any], List, EstimatorParams]] = None
    ) -> List[Dict[str, Any]]:
        if params is None:
            params = [{}]
        elif isinstance(params, EstimatorParams):
            if params.has_items:
                params = params.as_dict()["items"]
            else:
                params = [params.as_dict()]
        elif isinstance(params, dict):
            params = [params]
        return params

    params = _coerce_estimator_params(params)
    param_str = json.dumps(params)

    res_str = get_interpreter().estimate(entry_expr, param_str)
    res = json.loads(res_str)
    return EstimatorResult(res)


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


class StateDump:
    """
    A state dump returned from the Q# interpreter.
    """

    """
    The number of allocated qubits at the time of the dump.
    """
    qubit_count: int

    __inner: dict
    __data: StateDumpData

    def __init__(self, data: StateDumpData):
        self.__data = data
        self.__inner = data.get_dict()
        self.qubit_count = data.qubit_count

    def __getitem__(self, index: int) -> complex:
        return self.__inner.__getitem__(index)

    def __iter__(self):
        return self.__inner.__iter__()

    def __len__(self) -> int:
        return len(self.__inner)

    def __repr__(self) -> str:
        return self.__data.__repr__()

    def __str__(self) -> str:
        return self.__data.__str__()

    def _repr_markdown_(self) -> str:
        return self.__data._repr_markdown_()

    def check_eq(
        self, state: Union[Dict[int, complex], List[complex]], tolerance: float = 1e-10
    ) -> bool:
        """
        Checks if the state dump is equal to the given state. This is not mathematical equality,
        as the check ignores global phase.

        :param state: The state to check against, provided either as a dictionary of state indices to complex amplitudes,
            or as a list of real amplitudes.
        :param tolerance: The tolerance for the check. Defaults to 1e-10.
        """
        phase = None
        # Convert a dense list of real amplitudes to a dictionary of state indices to complex amplitudes
        if isinstance(state, list):
            state = {i: state[i] for i in range(len(state))}
        # Filter out zero states from the state dump and the given state based on tolerance
        state = {k: v for k, v in state.items() if abs(v) > tolerance}
        inner_state = {k: v for k, v in self.__inner.items() if abs(v) > tolerance}
        if len(state) != len(inner_state):
            return False
        for key in state:
            if key not in inner_state:
                return False
            if phase is None:
                # Calculate the phase based on the first state pair encountered.
                # Every pair of states after this must have the same phase for the states to be equivalent.
                phase = inner_state[key] / state[key]
            elif abs(phase - inner_state[key] / state[key]) > tolerance:
                # This pair of states does not have the same phase,
                # within tolerance, so the equivalence check fails.
                return False
        return True

    def as_dense_state(self) -> List[complex]:
        """
        Returns the state dump as a dense list of complex amplitudes. This will include zero amplitudes.
        """
        return [self.__inner.get(i, complex(0)) for i in range(2**self.qubit_count)]


def dump_machine() -> StateDump:
    """
    Returns the sparse state vector of the simulator as a StateDump object.

    :returns: The state of the simulator.
    """
    ipython_helper()
    return StateDump(get_interpreter().dump_machine())


def dump_circuit() -> Circuit:
    """
    Dumps the current circuit state of the interpreter.

    This circuit will contain the gates that have been applied
    in the simulator up to the current point.
    """
    ipython_helper()
    return get_interpreter().dump_circuit()
