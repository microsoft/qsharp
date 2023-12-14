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

    def _update_ui(self):
        self.buckets = self._new_buckets.copy()
        self.shot_count = self._new_count
        self._last_message = time.time()

    def _add_result(self, result):
        result_str = str(result["result"])
        old_value = self._new_buckets.get(result_str, 0)
        self._new_buckets.update({result_str: old_value + 1})
        self._new_count += 1

        # Only update the UI max 10 times per second
        if time.time() - self._last_message >= 0.1:
            self._update_ui()

    def __init__(self, results=None):
        super().__init__()

        self._new_buckets = {}
        self._new_count = 0
        self._last_message = time.time()

        # If provided a list of results, count the buckets and update.
        # Need to distinguish between the case where we're provided a list of results
        # or a list of ShotResults
        if results is not None:
            for result in results:
                if isinstance(result, dict) and "result" in result:
                    self._add_result(result)
                else:
                    # Convert the raw result to a ShotResult for the call
                    self._add_result({"result": result, "events": []})

            self._update_ui()

    def run(self, entry_expr, shots):
        import qsharp

        self._new_buckets = {}
        self._new_count = 0

        # Note: For now, we don't care about saving the results, just counting
        # up the results for each bucket. If/when we add output details and
        # navigation, then we'll need to save the results. However, we pass
        # 'save_results=True' to avoid printing to the console.
        qsharp.run(entry_expr, shots, on_result=self._add_result, save_events=True)

        # Update the UI one last time to make sure we show the final results
        self._update_ui()
