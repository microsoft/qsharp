### Copyright (c) Microsoft Corporation. All rights reserved.
### Licensed under the MIT License.

# Various examples to test operations using `dump_operation` python API

Testing operations in Q# can be done in Python using `dump_operation` API.
Modern QDK doesn't support native Q# tests unlike classic QDK, so we need a Python wrapper.

This sample outlines a multi-file Q# project that can be tested using Q#. This is organised as follows:
- src
    - BellState.qs
    - SWAP.qs
- qsharp.json
- README.md
- test_dump_operation.py

Reference Links:
- [Q# Testing guide](https://learn.microsoft.com/en-us/azure/quantum/user-guide/testing-debugging)
