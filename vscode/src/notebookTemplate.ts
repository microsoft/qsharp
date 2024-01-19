// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

// The VS Code notebook representation does not exactly match the ipynb JSON structure,
// so export a typed JSON object here as the representation to construct from.
export const notebookTemplate: vscode.NotebookData = {
  metadata: {},
  cells: [
    {
      kind: vscode.NotebookCellKind.Markup,
      languageId: "markdown",
      value: `## Azure Quantum Q# notebook

To use Jupyter Notebooks with Q#, the "qsharp" and "jupyterlab" Python packages should be installed.

To render charts as shown in this notebook, Matplotlib with notebook widgets support should be installed via "ipympl".

To submit to Azure Quantum as shown in this notebook, the "azure-quantum" package needs to be installed.

You can install all the above packages in your Python environment by running the below in your terminal:

\`\`\`bash
pip install jupyterlab qsharp ipympl azure-quantum
\`\`\``,
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "python",
      value: `# Run this cell first to enable the "%%qsharp" magic command used in later cells
import qsharp
`,
    },
    {
      kind: vscode.NotebookCellKind.Markup,
      languageId: "markdown",
      value:
        "The below cell shows Q# being written directly inside a Jupyter Notebook using the `%%qsharp` 'magic' command",
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "qsharp",
      value: `%%qsharp

// This makes the DumpMachine() function available.
open Microsoft.Quantum.Diagnostics;

operation RandomBit() : Result {
    // Qubits are only accesible for the duration of the scope where they
    // are allocated and are automatically released at the end of the scope.
    use qubit = Qubit();

    // Set the qubit in superposition by applying a Hadamard transformation.
    H(qubit);

    DumpMachine();

    // Measure the qubit. There is a 50% probability of measuring either
    // "Zero" or "One".
    let result = M(qubit);

    Message($"The result of the measurement is {result}.");

    // Reset the qubit so it can be safely released.
    Reset(qubit);
    return result;
}

// Call the operation we defined above.
RandomBit();
`,
    },
    {
      kind: vscode.NotebookCellKind.Markup,
      languageId: "markdown",
      value:
        "As well as invoking Q# operations immediately inline (as the last line in the cell above does), the defined operations can also be invoked from Python",
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "python",
      value: `r = qsharp.eval("RandomBit()")

if r == qsharp.Result.One:
    i = 1
else:
    i = 0

print(f"Python: Got integer {i}!")
`,
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "python",
      value: `%matplotlib widget

import matplotlib.pyplot as plt
import numpy as np
from collections import Counter

results = qsharp.run("RandomBit()", shots=1000)
# Sort the results so that the histogram labels appear in the correct order
results.sort()
# Count the number of times each result appears
counts = Counter(results)

(values, counts) = counts.keys(), counts.values()
xlabels = np.arange(len(counts))
plt.title("RandomBit() Results")
plt.bar(xlabels, counts)
plt.xticks(xlabels, values)
plt.show()
      `,
    },
    {
      kind: vscode.NotebookCellKind.Markup,
      languageId: "markdown",
      value: `## Submitting jobs to Azure Quantum

Different quantum hardware supports different capabilities, but all Azure Quantum providers support the 'base profile'
as defined in the 'Quantum Intermediate Representation' (QIR) specification. (For more details see <https://aka.ms/qdk.qir>)

To develop code using this base profile, reintialize the Q# compiler, connect to your Azure Quantum workspace, and submit the job.
`,
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "python",
      value: `# Reset the compiler to target the 'base profile' that all hardware supports

qsharp.init(target_profile=qsharp.TargetProfile.Base)
`,
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "python",
      value: "# WORKSPACE_CONNECTION_CODE",
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "qsharp",
      value: `%%qsharp

operation Random() : Result {
    use q = Qubit();
    H(q);
    let result = M(q);
    Reset(q);
    return result
}

operation RandomNBits(N: Int): Result[] {
    mutable results = [];
    for i in 0 .. N - 1 {
        let r = Random();
        set results += [r];
    }
    return results
}
`,
    },
    {
      kind: vscode.NotebookCellKind.Code,
      languageId: "python",
      value: `# Run the above code for 100 shots against the Rigetti simulator

operation = qsharp.compile("RandomNBits(4)")
target = workspace.get_targets("rigetti.sim.qvm")
job = target.submit(operation, "my-azure-quantum-job", shots=100)

# Wait for the job to complete
job.get_results()
`,
    },
  ],
};
