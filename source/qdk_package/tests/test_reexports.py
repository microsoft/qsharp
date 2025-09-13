# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest, importlib


def test_qdk_qsharp_submodule_available():
    qdk = importlib.import_module("qdk")
    assert hasattr(qdk, "qsharp"), "qdk.qsharp submodule not exposed"
    # Ensure a core API is reachable via submodule
    assert hasattr(qdk.qsharp, "run"), "qsharp.run missing in submodule"


def test_no_direct_run_or_estimate_exports():
    qdk = importlib.import_module("qdk")
    assert not hasattr(qdk, "run"), "'run' should not be directly exported"
    assert not hasattr(qdk, "estimate"), "'estimate' should not be directly exported"


def test_require_helper():
    qdk = importlib.import_module("qdk")
    mod = qdk.require("qsharp")
    assert hasattr(mod, "run")

    with pytest.raises(ImportError):
        qdk.require("__definitely_missing_feature__")


def test_azure_require_missing():
    qdk = importlib.import_module("qdk")

    # Should raise if azure extra not installed
    import importlib as _il

    try:
        _il.import_module("azure.quantum")
        installed = True
    except Exception:
        installed = False
    if not installed:
        with pytest.raises(ImportError):
            qdk.require("azure")


def test_qiskit_require_missing():
    qdk = importlib.import_module("qdk")

    try:
        importlib.import_module("qiskit")
        installed = True
    except Exception:
        installed = False
    if not installed:
        with pytest.raises(ImportError):
            qdk.require("qiskit")
