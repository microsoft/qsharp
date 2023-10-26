# Azure Quantum Development Kit (QDK) Preview

**This extension is currently in preview. If you are looking for the current stable QDK, see the documentation
at <https://learn.microsoft.com/en-us/azure/quantum/install-overview-qdk>**

_Note: To install pre-release versions of this extension, you must click the drop-down
next to the 'Install' button and select 'Install Pre-Release Version'_

This extension brings rich Q# language support to VS Code. Develop, build, and run your Q# code from VS Code either locally on simulators, or by submitting a job to Azure Quantum.

## Features

The Q# extension currently supports:

- Syntax highlighting and basic syntax features (e.g. brace matching)
- Q# cell support in Jupyter notebooks. The extension will detect `%%qsharp` magic cells and automatically update the cell language to Q#
- Error checking in Q# source files
- Breakpoint debugging and script execution for Q# source files
- Integration with Azure Quantum for job submission to quantum hardware providers
- Hover-definition and docs
- Go-to-definition
- Function signature help
- Snippet and sample support
- Completions

For more documentation and walkthroughs, see the wiki at <https://github.com/microsoft/qsharp/wiki>

## Contributing

To log issues, contribute to the project, or build the extension yourself, visit the repository at <https://github.com/microsoft/qsharp>

## Data and telemetry

This extension collects usage data and sends it to Microsoft to help improve our products and services.
Details of the telemetry sent can be seen in the source file at <https://github.com/microsoft/qsharp/blob/main/vscode/src/telemetry.ts>.
This extension respects the `telemetry.enableTelemetry` setting which you can learn more about at
<https://code.visualstudio.com/docs/supporting/faq#_how-to-disable-telemetry-reporting>.
