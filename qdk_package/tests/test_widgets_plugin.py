import importlib


def test_widgets_available_function():
    qdk = importlib.import_module("qdk")
    assert isinstance(qdk.widgets_available(), bool)


def test_widgets_module_import_when_installed(monkeypatch):
    # Simulate presence by creating a dummy module if not real
    import sys, types

    if "qsharp_widgets" not in sys.modules:
        dummy = types.ModuleType("qsharp_widgets")
        dummy.__dict__.update({"__version__": "0.test"})
        sys.modules["qsharp_widgets"] = dummy
    qdk_widgets = importlib.import_module("qdk.widgets")
    assert hasattr(qdk_widgets, "__doc__")
