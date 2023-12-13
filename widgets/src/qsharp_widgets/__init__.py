# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import importlib.metadata
import pathlib
import time

import anywidget
import traitlets

try:
    __version__ = importlib.metadata.version("qsharp_widgets")
except importlib.metadata.PackageNotFoundError:
    __version__ = "unknown"


class SpaceChart(anywidget.AnyWidget):
    _esm = pathlib.Path(__file__).parent / "static" / "index.js"
    _css = pathlib.Path(__file__).parent / "static" / "index.css"

    comp = traitlets.Unicode("SpaceChart").tag(sync=True)
    estimates = traitlets.Dict().tag(sync=True)

    def __init__(self, estimates):
        super().__init__()
        self.estimates = estimates


class EstimateDetails(anywidget.AnyWidget):
    _esm = pathlib.Path(__file__).parent / "static" / "index.js"
    _css = pathlib.Path(__file__).parent / "static" / "index.css"

    comp = traitlets.Unicode("EstimateDetails").tag(sync=True)
    estimates = traitlets.Dict().tag(sync=True)

    def __init__(self, estimates):
        super().__init__()
        self.estimates = estimates


class Histogram(anywidget.AnyWidget):
    _esm = pathlib.Path(__file__).parent / "static" / "index.js"
    _css = pathlib.Path(__file__).parent / "static" / "index.css"

    comp = traitlets.Unicode("Histogram").tag(sync=True)
    buckets = traitlets.Dict().tag(sync=True)
    shot_count = traitlets.Integer().tag(sync=True)

    _new_buckets = {}
    _new_count = 0
    _last_message = time.time()

    results = []

    def _update_ui(self):
        self.buckets = self._new_buckets.copy()
        self.shot_count = self._new_count
        self._last_message = time.time()

    def add_result(self, result):
        result_str = str(result["result"])
        old_value = self._new_buckets.get(result_str, 0)
        self._new_buckets.update({result_str: old_value + 1})
        self._new_count += 1

        # Only update the UI max 20 times per second
        if time.time() - self._last_message >= 0.05:
            self._update_ui()

    def __init__(self, entry_expr=None, shot_count=None):
        super().__init__()
        if entry_expr and shot_count:
            self.run(entry_expr, shot_count)

    def run(self, expr, shots):
        import qsharp

        self._new_buckets = {}
        self._new_count = 0
        self.results = qsharp.run_histogram(expr, shots, self.add_result)
        self._update_ui()
