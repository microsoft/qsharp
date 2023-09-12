# Q# Language Support for VS Code

_This extension is an unstable product that is still under active development. We do not recommend relying on this in production environments._

This extension brings rich Q# language support to VSCode. Develop, build, and run your Q# code from VSCode either locally on simulators, or by submitting a job to Azure Quantum. 

## Features

The Q# extension currently supports:
- Syntax highlighting and basic syntax features (e.g. brace matching)
- Q# cell support in Jupyter notebooks. The extension will detect `%%qsharp` magic cells and automatically update the cell language to Q#
- Error checking in Q# source files
- Breakpoint debugging and script execution for Q# source files
- Integration with Azure Quantum for quantum job submission
- Hover-definition and docs
- Go-to-definition
- Function signature help
- Snippet and sample support
- Completions

## Building the Extension Locally
To build the extension locally, see [BUILDING.md](https://github.com/microsoft/qsharp/blob/main/vscode/BUILDING.md).