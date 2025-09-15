try:
    from azure.quantum import Workspace

    # TODO: Other re-exports from submodules
except Exception as ex:
    raise ImportError(
        'qdk.azure requires the azure extra. Install with: pip install "qdk[azure]"'
    ) from ex
