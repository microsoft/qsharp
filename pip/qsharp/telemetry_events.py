# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from .telemetry import log_telemetry
import math

# TODO: This should be populated by the build in the main module
QSHARP_VERSION = "0.0.0"

# TODO: Log extra params like qubit count (buckets), qubit type for RE, etc.?

default_props = {"qsharp.version": QSHARP_VERSION}

# For metrics such as duration, we want to capture things like how many shots or qubits in
# the additional properties. However properties shouldn't be 'continuous' values, as they
# create new 'dimensions' on the backend, which is limited, thus we want to bucket these properties.

# See some of the notes at: https://learn.microsoft.com/en-us/azure/azure-monitor/essentials/metrics-custom-overview#design-limitations-and-considerations


def get_shots_bucket(shots: int) -> int:
    if shots <= 1:
        return 1
    elif shots >= 1000000:
        # Limit the buckets upper bound
        return 1000000
    else:
        # Bucket into nearest (rounded up) power of 10, e.g. 75 -> 100, 450 -> 1000, etc.
        return 10 ** math.ceil(math.log10(shots))


# gets the order of magnitude for the number of qubits
def get_qubits_bucket(qubits: int) -> int:
    if qubits <= 1:
        return 1
    elif qubits >= 1000000:
        # Limit the buckets upper bound
        return 1000000
    else:
        # Bucket into nearest (rounded up) power of 10, e.g. 75 -> 100, 450 -> 1000, etc.
        return 10 ** math.ceil(math.log10(qubits))


def on_import() -> None:
    log_telemetry("qsharp.import", 1, properties=default_props)


def on_run(shots: int) -> None:
    log_telemetry(
        "qsharp.run",
        1,
        properties={**default_props, "shots": get_shots_bucket(shots)},
    )


def on_run_end(durationMs: float, shots: int) -> None:
    log_telemetry(
        "qsharp.run.durationMs",
        durationMs,
        properties={**default_props, "shots": get_shots_bucket(shots)},
        type="histogram",
    )


def on_compile(profile: str) -> None:
    log_telemetry("qsharp.compile", 1, properties={**default_props, "profile": profile})


def on_compile_end(durationMs: float, profile: str) -> None:
    log_telemetry(
        "qsharp.compile.durationMs",
        durationMs,
        properties={**default_props, "profile": profile},
        type="histogram",
    )


def on_estimate(qubits: int) -> None:
    log_telemetry(
        "qsharp.estimate",
        1,
        properties={**default_props, "qubits": get_qubits_bucket(qubits)},
    )


def on_estimate_end(durationMs: float) -> None:
    log_telemetry(
        "qsharp.estimate.durationMs",
        durationMs,
        properties=default_props,
        type="histogram",
    )
