# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import importlib.metadata
import pathlib

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
