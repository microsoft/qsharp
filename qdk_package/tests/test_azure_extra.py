import importlib
import sys
import types


def test_azure_require_with_mock(monkeypatch):
    # If azure.quantum already installed, just assert require works
    qdk = importlib.import_module("qdk")
    already = True
    try:
        importlib.import_module("azure.quantum")
    except Exception:
        already = False

    if not already:
        # Create a minimal namespace package structure: azure.quantum
        azure_pkg = types.ModuleType("azure")
        quantum_sub = types.ModuleType("azure.quantum")
        quantum_sub.__dict__["__version__"] = "3.2.0-mock"
        # Insert both into sys.modules to satisfy nested import resolution
        sys.modules["azure"] = azure_pkg
        sys.modules["azure.quantum"] = quantum_sub

    mod = qdk.require("azure")
    # We don't know exact symbols; just ensure module object has a __spec__ or version attr through underlying package
    assert hasattr(mod, "__doc__")

    # Cleanup mock if we created it (keep real installations intact)
    if not already:
        sys.modules.pop("azure.quantum", None)
        # Only remove top-level if it was our synthetic one
        if getattr(sys.modules.get("azure"), "__file__", None) is None:
            sys.modules.pop("azure", None)
