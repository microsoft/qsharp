# qdk

Experimental meta-package for the Quantum Development Kit (QDK) that bundles the existing
`qsharp` Python package together with optional extras under a single, stable import root: `import qdk`.

The design is intentionally minimal: submodules plus import-time detection of optional components.

## Install

Base (always includes `qsharp`):

```bash
pip install qdk
```

Jupyter extra (bundles widgets + JupyterLab extension package — provides only the `qdk.widgets` Python surface):

```bash
pip install qdk[jupyter]
```

Azure Quantum extra (adds `azure-quantum`):

```bash
pip install qdk[azure]
```

Qiskit extra (adds `qiskit`):

```bash
pip install qdk[qiskit]
```

All extras:

```bash
pip install qdk[all]
```

## Quick Start

```python
from qdk import qsharp

result = qsharp.run("operation Hello() : Result { use q = Qubit(); H(q); return M(q); }")
print(result)
```

Widgets (installed via jupyter extra):

```python
from qdk import widgets_available, require

if widgets_available():
    widgets = require("widgets")
    # Use widgets per qsharp-widgets documentation
```

Azure Quantum (if installed):
Qiskit (if installed):

```python
from qdk import qiskit_available, require

if qiskit_available():
    qk = require("qiskit")
    # Example: qk.transpile(...)
```

```python
from qdk import azure_available, require

if azure_available():
    azure = require("azure")
    # Example: azure.Workspace(...) etc., per azure-quantum docs
```

## Public API Surface

Root-level symbols (kept intentionally small):

| Symbol                | Description                                                                 |
| --------------------- | --------------------------------------------------------------------------- |
| `qsharp`              | Submodule re-export of the upstream `qsharp` package.                       |
| `widgets_available()` | Boolean: is widget support (jupyter extra) installed?                       |
| `azure_available()`   | Boolean: is the azure extra installed?                                      |
| `qiskit_available()`  | Boolean: is the qiskit extra installed?                                     |
| `require(name)`       | Retrieve a feature module (`"qsharp"`, `"widgets"`, `"azure"`, `"qiskit"`). |

Submodules:

- `qdk.qsharp` – direct passthrough to `qsharp` APIs.
- `qdk.widgets` – only if `qsharp-widgets` installed (through `qdk[jupyter]`).
- `qdk.azure` – only if `azure-quantum` installed.
- `qdk.qiskit` – only if `qiskit` installed.
- `qdk.estimator` – shim re-export of `qsharp.estimator` (always present if underlying `qsharp` provides it).
- `qdk.openqasm` – shim re-export of `qsharp.openqasm` for OpenQASM integration.

## `require()` Helper

```python
from qdk import require

# Core always available (raises only if qsharp itself missing):
core = require("qsharp")

# Widgets / jupyter (alias) extra
try:
    widgets = require("widgets")  # or require("jupyter")
except ImportError:
    widgets = None

# Azure extra
try:
    azure = require("azure")
except ImportError:
    azure = None

# Qiskit extra
try:
    qk = require("qiskit")
except ImportError:
    qk = None

# Estimator & OpenQASM shims (will ImportError only if that part of qsharp missing)
from qdk import estimator, openqasm
```

## Design Notes

- Root re-exports selected utility symbols from `qsharp` (e.g. `code`, `set_quantum_seed`, types) for convenience; algorithm APIs still live under `qdk.qsharp`.
- Additional shims (`qdk.estimator`, `qdk.openqasm`) are thin pass-throughs to the corresponding `qsharp` submodules for discoverability.
- Optional extras are thin pass-through modules; failure messages instruct how to install.
- Tests may stub dependencies in isolation environments.

## Testing

The test suite validates packaging & import contract without requiring the real
optional dependencies to be installed.

Current approach (kept intentionally lean):

1. Core behavior: ensure the root package exposes only the minimal public API and that
   `require()` returns the expected submodules.
2. A lightweight stub for the upstream `qsharp` package is injected (see `tests/conftest.py`)
   if the true package is not present, enabling fast iteration when working only on this meta-package.
3. Optional extras (widgets, azure, qiskit) are tested using synthetic modules created in `tests/mocks.py`:
   - `mock_widgets()` creates a lightweight `qsharp_widgets` module (with a version attribute). Tests assert the `qdk.widgets` shim imports (doc presence).
   - `mock_azure()` creates the nested namespace `azure.quantum` (with a version attribute). Tests assert the `qdk.azure` shim imports (doc presence).
   - `mock_qiskit()` creates a `qiskit` module exposing a callable `transpile()` so tests can assert a functional symbol survives re-export.
4. No network or cloud interactions are performed; all tests operate purely on import mechanics and mocks.

### Running the tests

Install test tooling:

```bash
python -m pip install pytest
python -m pytest -q qdk_package/tests
```

Because mocks are used, failures generally indicate packaging / import logic regressions
rather than upstream functional issues with the real dependencies.
