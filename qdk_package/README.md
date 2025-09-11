# qdk

Experimental meta-package for the Quantum Development Kit (QDK) that bundles the existing
`qsharp` Python package together with optional extras under a single, stable import root: `import qdk`.

The design is intentionally minimal: submodules plus import-time detection of optional components.

## Rationale

- Provide a future-facing namespace (`qdk`) without forcing an immediate rename or massive re-export surface.
- Encourage explicit submodule usage (`qdk.qsharp`, `qdk.widgets`, `qdk.azure`) rather than star imports into the root.
- Make optional features invisible unless their dependency is installed.

## Install

Base (always includes `qsharp`):

```bash
pip install qdk
```

Widgets extra (adds `qsharp-widgets`):

```bash
pip install qdk[widgets]
```

Azure Quantum extra (adds `azure-quantum`):

```bash
pip install qdk[azure]
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

Widgets (if installed):

```python
from qdk import widgets_available, require

if widgets_available():
    widgets = require("widgets")
    # Use widgets per qsharp-widgets documentation
```

Azure Quantum (if installed):

```python
from qdk import azure_available, require

if azure_available():
    azure = require("azure")
    # Example: azure.Workspace(...) etc., per azure-quantum docs
```

## Public API Surface

Root-level symbols (kept intentionally small):

| Symbol                | Description                                                     |
| --------------------- | --------------------------------------------------------------- |
| `qsharp`              | Submodule re-export of the upstream `qsharp` package.           |
| `widgets_available()` | Boolean: is the widgets extra installed?                        |
| `azure_available()`   | Boolean: is the azure extra installed?                          |
| `require(name)`       | Retrieve a feature module (`"qsharp"`, `"widgets"`, `"azure"`). |

Submodules:

- `qdk.qsharp` – direct passthrough to `qsharp` APIs.
- `qdk.widgets` – only if `qsharp-widgets` installed.
- `qdk.azure` – only if `azure-quantum` installed.

## `require()` Helper

```python
from qdk import require
core = require("qsharp")
try:
    azure = require("azure")
except ImportError:
    azure = None
```

## Design Notes

- No implicit re-export of individual functions (e.g. `run`) at the root.
- Optional extras are thin pass-through modules; failure messages instruct how to install.
- Tests may stub dependencies in isolation environments.

## Testing

Local test strategy focuses on exercising the meta-package logic without requiring
all optional dependencies to be installed:

1. Core tests import `qdk` and verify that only submodules (not individual functions)
   appear at the root.
2. A lightweight stub of the upstream `qsharp` package is injected in test environments
   if the real package is absent (see `tests/conftest.py`). This allows running tests
   quickly while developing the meta-package itself.
3. Optional extras are validated in two modes:
   - Absence path: asserting `require("widgets")` / `require("azure")` raises `ImportError`.
   - Presence path: synthetic modules are inserted into `sys.modules` (e.g. a mock
     `azure.quantum`) so `require("azure")` succeeds without the actual dependency.
4. No network calls or cloud resources are touched; Azure functionality is not
   exercised beyond import and basic attribute existence.

### Running the tests

Install dev tooling (if you add a `dev` extra you could do `pip install qdk[dev]`):

```bash
python -m pip install -r requirements.txt  # if you later add one
python -m pip install pytest
python -m pytest -q qdk_package/tests
```

Because mocks are used, failures generally indicate packaging / import logic regressions
rather than upstream functional issues.

## Roadmap / Possible Extensions

- Additional extras (interop, advanced visualization) as they stabilize.
- Potential environment introspection helper (e.g. an `info` command) later.

## Status

Prototype; expect adjustments before any stable (1.0) release.

## License

See `LICENSE.txt`.
