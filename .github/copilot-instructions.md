# Copilot instructions for the `qsharp` repo

## Overview

This repo contains the Microsoft Quantum Development Kit (QDK), which provides tooling for the Q# language. Q# is an open-source programming language designed for developing and running quantum algorithms. This repo is publicly available at https://github.com/microsoft/qsharp .

## Architecture

Most of the core components are implemented in Rust. These components are packaged in two ways:

1. Compiled as a native Python module and packaged into the `qsharp` Python package
2. Compiled into WebAssembly and packaged into the `qsharp-lang` npm package

## Repo Contents

**Rust**

- **allocator/**: A copy of `mimalloc`, used for memory allocation in the Rust components of the QDK
- **compiler/**: Core compiler and language processing components
  - **qsc/**: Core compiler logic and command-line tooling
  - **qsc_ast/**: Abstract syntax tree definition and utilities
  - **qsc_circuit/**: Circuit diagram representation and generation
  - **qsc_codegen/**: Code generation utilities (QIR, Q#)
  - **qsc_data_structures/**: Common data structures used by the compiler
  - **qsc_doc_gen/**: Documentation generation tools
  - **qsc_eval/**: Runtime evaluation and simulation
  - **qsc_fir/**: Flat IR
  - **qsc_formatter/**: Q# code formatter
  - **qsc_frontend/**: Compiler frontend components
  - **qsc_hir/**: High-level Intermediate Representation
  - **qsc_linter/**: Code quality and style checking
  - **qsc_lowerer/**: IR lowering transformations
  - **qsc_parse/**: Q# parser
  - **qsc_partial_eval/**: Partial evaluation and optimization
  - **qsc_passes/**: HIR passes
  - **qsc_project/**: Project system and manifest handling
  - **qsc_qasm/**: OpenQASM to Q# compiler
  - **qsc_qasm3/**: OpenQASM 3.0 support
  - **qsc_rca/**: Resource counting and analysis
  - **qsc_rir/**: Runtime Intermediate Representation
- **fuzz/**: Fuzz testing infrastructure for the compiler
- **language_service/**: Q# language service for editor features
  - **src/code_action.rs**: Code actions
  - **src/code_lens/**: Code lens
  - **src/completion/**: Completions
  - **src/definition/**: Go to definition
  - **src/hover/**: Hover
  - **src/references/**: Find references
  - **src/rename/**: Inline rename
  - **src/signature_help/**: Signature help
  - **src/state/**: Language service state management
- **noisy_simulator/**: Simulator for quantum noise modeling
- **resource_estimator/**: Quantum Resource Estimator implementation
- **wasm/**: WebAssembly bindings for core components

**Build & Release**
- **docker/**: Docker container configurations

**Python**

- **pip/**: The `qsharp` Python package
  - **qsharp/**: Python package source
  - **src/**: Rust implementation for Python bindings
  - **tests/**: Unit tests
  - **tests-integration/**: Integration tests with Qiskit, PyQIR, QIR Runner, and simulators
    - **interop_qiskit/**: Qiskit interoperability tests
      - **test_circuits/**: Test circuit definitions 
      - **test_gateset_qasm.py**: QASM gate set tests
      - **test_gate_correctness.py**: Gate correctness validation
      - **test_qir.py**: QIR integration with Qiskit
      - **test_qsharp.py**: Q# and Qiskit interop tests
      - **test_re.py**: Resource estimator tests
      - **test_run_sim.py**: Simulation runtime tests
    - **resources/**: Test resources for adaptive QIR
    - **test_adaptive_ri_qir.py**: Adaptive Result Interop QIR tests
    - **test_adaptive_rif_qir.py**: Adaptive Result+Feedback Interop QIR tests
    - **test_base_qir.py**: Base QIR functionality tests
- **jupyterlab/**: JupyterLab extension for Q#
- **widgets/**: Q# Jupyter widgets

**Q# Content**

- **katas/**: Quantum computing tutorials and exercises
- **library/**: Q# standard and domain-specific libraries
  - **chemistry/**: Chemistry-specific quantum operations
  - **core/**: Q# core library
  - **fixed_point/**: Fixed-point arithmetic support
  - **qtest/**: Testing utilities
  - **rotations/**: Quantum rotation operations
  - **signed/**: Signed arithmetic operations
  - **std/**: Q# standard library
- **samples/**: Example Q# programs demonstrating language features
- **samples_test/**: Tests for sample code

**JavaScript/TypeScript**

- **npm/**: The `qsharp-lang` npm package
- **playground/**: Q# Playground website
- **vscode/**: Visual Studio Code extension for Q#
  - **src/**: Extension source code
    - **language-service/**: Language service integration
      - **activate.ts**: Language service activation
      - **codeActions.ts**: Code actions implementation
      - **codeLens.ts**: Code lens provider
      - **completion.ts**: Autocompletion provider
      - **definition.ts**: Go to definition
      - **diagnostics.ts**: Diagnostics handling
      - **format.ts**: Code formatting
      - **hover.ts**: Hover information provider
      - **notebook.ts**: Notebook integration
      - **references.ts**: Find references
      - **rename.ts**: Symbol renaming
      - **signature.ts**: Signature help provider
      - **testExplorer.ts**: Test explorer integration
    - **webview/**: UI components
      - **docview.tsx**: Documentation view
      - **editor.tsx**: Editor components
      - **help.tsx**: Help view
      - **webview.tsx**: Webview base
      - **theme.ts**: Theme handling
    - **azure/**: Azure integration
    - **debugger/**: Debugger implementation
    - **copilot/**: Copilot integration
    - **gh-copilot/**: GitHub Copilot integration
    - **extension.ts**: Main extension entry point
    - **circuit.ts**: Circuit representation
    - **circuitEditor.ts**: Circuit editor
    - **diagnostics.ts**: Diagnostic handling
    - **projectSystem.ts**: Project management
    - **qirGeneration.ts**: QIR generation
    - **notebook.ts**: Notebook integration
    - **telemetry.ts**: Telemetry collection
  - **resources/**: Extension resources
  - **syntaxes/**: Q# syntax definitions
  - **test/**: Extension tests
  - **wasm/**: WebAssembly integration

## Development workflow

- `./build.py` runs full CI checks, including lints and unit tests.
- `./build.py --wasm --npm --vscode` to only build the VS Code extension, including its dependencies the WASM module and the `qsharp-lang` npm package.
- `./build.py --pip` to only build the `qsharp` Python package.
- When working in Rust parts of the codebase, using `cargo` commands is usually more efficient than building via `./build.py`.
  - Many lints can be auto-fixed via `cargo clippy --fix`.
- When working in JavaScript/TypeScript parts of the codebase, using `npm` commands is usually more efficient than building via `./build.py`.

## Coding Standards

Before opening a PR, ensure the following.

- Code **must** be formatted by running `cargo fmt` and `npm run prettier:fix`.
- `./build.py` without any command-line arguments **must** run without errors or warnings.
