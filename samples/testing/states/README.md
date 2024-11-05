# Testing Quantum States

This sample project demonstrates testing Q# code that has to end up with a certain quantum state.

## Testing Method

The most convenient way to validate the quantum state of the program in Q# is using `dump_machine` Python API.

1. **Check that the amplitudes match the dense array of expected amplitudes:**  
   Use the `as_dense_state()` method of `StateDump` class to convert it to an array of amplitudes and compare it with the expected one.

2. **Check that the state matches the expected one up to a global phase:**  
   Use the `check_eq()`  method of `StateDump` class to compare it to the given array of amplitudes, taking into account the possible global phase difference.


## Project Structure

This sample project is a multi-file Q# project that showcases both testing methods. The project structure is as follows:

- src
    - `StatePrep.qs`: Q# file containing the state preparation operations to be tested.
- `qsharp.json`: Q# project manifest file, instructing the compiler to include all files in `src` directory.
- `test_states.py`: Python wrapper containing tests.
