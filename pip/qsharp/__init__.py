# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import (
    init,
    eval,
    run,
    compile,
    estimate,
    dump_machine,
)

from ._native import Result, Pauli, QSharpError, TargetProfile, StateDump

# IPython notebook specific features
try:
    if __IPYTHON__:  # type: ignore
        from ._ipython import register_magic, enable_classic_notebook_codemirror_mode

        register_magic()
        enable_classic_notebook_codemirror_mode()
except NameError:
    pass


__all__ = [
    "init",
    "eval",
    "run",
    "dump_machine",
    "compile",
    "estimate",
    "Result",
    "Pauli",
    "QSharpError",
    "TargetProfile",
    "StateDump",
]

import importlib.metadata
__version__ = importlib.metadata.version("qsharp-lang")

# Check if there is a newer published version of the package
# and warn the user if there is. This is best effort only.

try:
    import urllib.request
    import json
    import sys
    from packaging.version import parse as parse_version

    url = 'https://pypi.org/pypi/qsharp-lang/json'
    with urllib.request.urlopen(url, timeout=3) as f:
        pypi_info = json.loads(f.read().decode('utf-8'))
    latest_version = pypi_info['info']['version']

    if parse_version(latest_version) > parse_version(__version__):
        print(
            f'You are using qsharp {__version__}, but version {latest_version} is available.\n'
            'Consider upgrading via the "pip install --upgrade qsharp" command.',
            file=sys.stderr)
except:
    pass
