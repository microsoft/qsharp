# Q# Language Support for Python

Q# is an open-source, high-level programming language for developing and running quantum algorithms.
The `qsharp-lang` package for Python provides interoperability with the Q# interpreter, making it easy
to simulate Q# programs within Python.

## Installation

To install the Q# language package, run:

```bash
pip install qsharp-lang
```

## Usage

First, import the `qsharp` module:

```python
import qsharp
```

Then, use the `%%qsharp` cell magic to run Q# directly in Jupyter notebook cells:

```qsharp
%%qsharp

open Microsoft.Quantum.Diagnostics;

@EntryPoint()
operation BellState() : Unit {
    use qs = Qubit[2];
    H(qs[0]);
    CNOT(qs[0], qs[1]);
    DumpMachine();
    ResetAll(qs);
}

BellState()
```

## Support

For more documentation and to browse issues, please visit the Q# project wiki at [https://github.com/microsoft/qsharp/wiki].

## Contributing

Q# welcomes your contributions! Visit the Q# GitHub repository at [https://github.com/microsoft/qsharp] to find out more about the project.
