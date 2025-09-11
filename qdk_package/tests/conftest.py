# Ensure the local 'src' directory is on sys.path so 'qdk' can be imported without installation.
import sys
from pathlib import Path
import types

# Ensure local src on path for importing 'qdk'.
_src = Path(__file__).resolve().parents[1] / "src"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))

# Provide a lightweight stub for 'qsharp' if it is not installed in the env.
try:  # pragma: no cover - only runs if real package present
    import qsharp  # type: ignore
except Exception:  # pragma: no cover - create stub
    stub = types.ModuleType("qsharp")

    def _not_impl(*a, **k):  # minimal placeholder functions
        raise NotImplementedError("qsharp stub: real 'qsharp' package not installed")

    # Minimal API surface referenced by tests / features list
    stub.run = _not_impl
    stub.estimate = _not_impl
    stub.__all__ = ["run", "estimate"]
    sys.modules["qsharp"] = stub
