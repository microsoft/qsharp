# Testing Operations in the QDK
This sample project demonstrates different approaches to testing operations in the QDK, both via Python and within Q# itself.

## Testing Methods

There are two primary ways to test operations in the QDK:

1. **Dumping Operation Output:**
   - Use the `dump_operation` Python API to retrieve the operation's representation and compare it against the expected output.

2. **Q# `Fact` Assertions:**
   - Define a `Fact` function in your Q# code that uses the `CheckOperationsAreEqual` operation to verify if two operations are identical. The `Fact` function asserts that the check returns `true`.

## Project Structure
This sample project is a multi-file Q# project that showcases both testing methods. The project structure is as follows:

- src
    - "BellState.qs": Q# file containing the `AllBellStates` operation to be tested
    - "Test_SWAP.qs": Q# file containing the `ApplySWAP1` and `ApplySWAP2` operations to be tested
    - "OperationEquivalence.qs": Q# file containing the `TestEquivalence` operation to be called in python wrapper
- "qsharp.json": Q# project configuration file
- test_dump_operation.py: "Python wrapper containing tests"

## Installation
- Install the `qsharp` python package by following the instructions mentioned [here](https://learn.microsoft.com/azure/quantum/install-overview-qdk#add-support-for-python-and-jupyter-notebooks).
- Install `pytest` python package.

## Running the sample
Open the `samples/testing/dump_operation` directory, and run `pytest` command.

## Reference Links:
- [Q# Testing guide](https://learn.microsoft.com/azure/quantum/user-guide/testing-debugging).
- [Getting started with the QDK](https://learn.microsoft.com/azure/quantum/install-overview-qdk)
- [Getting started with Pytest](https://docs.pytest.org/en/stable/getting-started.html)
