# Testing Classical Return Values

This sample project demonstrates testing Q# code that returns classical values.

## Testing Methods

There are two primary ways to test classical values in Q#:

1. **Return them to Python and run checks in Python:**  
   Use the `eval` Python API to get the results of Q# code and check that they are as expected.

2. **Q# `Fact` Assertions:**  
   Use a `Fact` function in your Q# code that checks whether the classical value within it are correct. The `Fact` function asserts that the check returns `true`.

## Project Structure
This sample project is a multi-file Q# project that showcases both testing methods. The project structure is as follows:

- src
    - `ClassicalFunction.qs`: Q# file containing the classical function to be tested
    - `Measurement.qs`: Q# file containing the operation with `Result[]` return type to be tested
    - `TestCode.qs`: Q# file containing the test logic for the first two files to be called in Python wrapper
- `qsharp.json`: Q# project manifest file, instructing compiler to include all files in `src` directory.
- `test_classical_values.py`: Python wrapper containing tests.
