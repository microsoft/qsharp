import importlib
import types

import pytest

from mocks import (
    mock_widgets,
    mock_azure,
    mock_qiskit,
    cleanup_modules,
)


# Standard contract description for each extra we test.
EXTRAS = {
    "widgets": {
        "mock": mock_widgets,
        "require_name": "widgets",
        "module": "qdk.widgets",
        "post_assert": lambda mod: hasattr(mod, "__doc__"),
    },
    "azure": {
        "mock": mock_azure,
        "require_name": "azure",
        "module": "qdk.azure",
        # azure is nested (azure.quantum). We just assert the exported module has a __doc__.
        "post_assert": lambda mod: hasattr(mod, "__doc__"),
    },
    "qiskit": {
        "mock": mock_qiskit,
        "require_name": "qiskit",
        "module": "qdk.qiskit",
        "post_assert": lambda mod: hasattr(mod, "transpile"),
    },
}


@pytest.mark.parametrize("extra_key", EXTRAS.keys())
def test_availability_function_shape(extra_key):
    qdk = importlib.import_module("qdk")
    # widgets_available / azure_available / qiskit_available share naming pattern
    attr = f"{extra_key}_available"
    if hasattr(qdk, attr):
        value = getattr(qdk, attr)()
        assert isinstance(value, bool)
    else:
        # If availability helper missing that's a test design issue
        pytest.fail(f"Missing availability helper {attr}")


@pytest.mark.parametrize("extra_key", EXTRAS.keys())
def test_require_with_mock(extra_key):
    qdk = importlib.import_module("qdk")
    spec = EXTRAS[extra_key]
    created = spec["mock"]()
    try:
        mod = qdk.require(spec["require_name"])
        assert spec["post_assert"](mod)
    finally:
        cleanup_modules(created)


@pytest.mark.parametrize("extra_key", EXTRAS.keys())
def test_direct_module_import_with_mock(extra_key):
    spec = EXTRAS[extra_key]
    created = spec["mock"]()
    try:
        imported = importlib.import_module(spec["module"])
        assert spec["post_assert"](imported)
    finally:
        cleanup_modules(created)
