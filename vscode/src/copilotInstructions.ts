// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "../../npm/qsharp/dist/log";

/**
 * Command to update or create the copilot instructions file for Q#.
 * Shows a prompt to the user and updates the file if confirmed.
 */
export async function updateGhCopilotInstructionsCommand() {
  // Show a yes/no prompt to the user
  const response = await vscode.window.showInformationMessage(
    "We're about to update your `copilot-instructions.md` file.\n\n" +
      "This file helps GitHub Copilot understand and work better with Q# files and features provided by the Quantum Development Kit extension.\n\n" +
      "Would you like to proceed with updating `copilot-instructions.md`?",
    { modal: true },
    "Yes",
    "No",
  );

  if (response !== "Yes") {
    return; // User canceled or dismissed the dialog
  }

  // Ensure the .github directory exists in the workspace
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    vscode.window.showErrorMessage("No workspace folder is open");
    return;
  }

  const workspaceFolder = workspaceFolders[0].uri;

  try {
    const githubDir = vscode.Uri.joinPath(workspaceFolder, ".github");
    const instructionsFile = vscode.Uri.joinPath(
      githubDir,
      "copilot-instructions.md",
    );

    const fileContent = `# Q# coding instructions

Follow these instructions when generating Q# code in .qs files,
and Q# project folders tha include a \`qsharp.json\` file.

## Testing

The Q# language supports unit testing in VS Code. To write a test, use the \`@Test()\`
attribute on an operation, and \`fail\` with a message on test failure, e.g.,

\`\`\`qsharp
@Test()
operation MyTestCase() : Unit {
    let result = DoOp();
    if (result != Expected) {
        fail $"DoOp returned {result}"
    }
}
\`\`\`

Note: Prefer using a conditional \`fail\` statement to \`Fact\` calls, as \`fail\` gives a better error location.

## Syntax

When writing Q#, the \`for (i in 1..100)\` syntax is invalid, use \`for i in 1..100\` or
\`for element in array[2...]\`.

While many Q# operators are C-like, it uses \`or\` instead of \`||\` and \`and\` instead of \`&&\`.

## Multi-file projects

Modern Q# does not use \`namespace\` blocks to enclose code.
When Q# code is in different files in a project, each function or operation is in a namespace
which is the name of the containing file. For example, if \`Main.qs\` has an operation \`Foo\`,
then \`Tests.qs\` could reference the operation as \`Main.Foo\`, or bring \`Foo\` into scope by
adding \`import Main.Foo;\` in the file.

## Libraries

A Q# project can reference a library from GitHub but updating the \`dependencies\` entry of
the \`qsharp.json\` file. For example, to reference the \`chemistry\` library, the \`qsharp.json\`
file might appear as:

\`\`\`json
{
    "dependencies": {
        "Chemistry": {
            "github": {
                "ref": "v1.15.0",
                "owner": "microsoft",
                "repo": "qsharp",
                "path": "library/chemistry"
            }
        }
    }
}
\`\`\`

## Jupyter Notebooks

Q# has first-class support for Jupyter notebooks. Typically the first cell will contain \`import qsharp\`.

Jupyter cells can contain Q# code directly by using the \`%%qsharp\` magic command at the beginning of the cell. For example:

\`\`\`python
%%qsharp

operation GHZSample(n: Int) : Result[] {
    use qs = Qubit[n];

    H(qs[0]);
    ApplyToEach(CNOT(qs[0], _), qs[1...]);

    let results = MeasureEachZ(qs);
    ResetAll(qs);
    return results;
}
\`\`\`

The \`qsharp_widgets\` package provides viewers for circuits and histograms, e.g.

\`\`\`python
from qsharp_widgets import Circuit, Histogram
Circuit(qsharp.circuit("GHZSample(3)"))
\`\`\`

Note that the latest Q# and QDK releases don't require or use the old IQ# kernel. It just needs to the \`qsharp\` PyPI package,
and maybe \`qsharp_widgets\` for visuals.`;

    // Check if .github directory exists, create if it doesn't
    try {
      await vscode.workspace.fs.stat(githubDir);
    } catch {
      // Directory doesn't exist, create it
      await vscode.workspace.fs.createDirectory(githubDir);
    }

    // Check if the file already exists
    try {
      const existingContent =
        await vscode.workspace.fs.readFile(instructionsFile);
      const existingText = new TextDecoder("utf-8").decode(existingContent);

      if (!existingText.includes(fileContent.trim())) {
        // Only append if the content isn't already there
        const encoder = new TextEncoder();
        const updatedContent = existingText + "\n\n" + fileContent;
        await vscode.workspace.fs.writeFile(
          instructionsFile,
          encoder.encode(updatedContent),
        );
        vscode.window.showInformationMessage(
          "Successfully updated copilot-instructions.md",
        );
      } else {
        vscode.window.showInformationMessage(
          "copilot-instructions.md already contains Q# instructions",
        );
      }
    } catch {
      // File doesn't exist, create it
      const encoder = new TextEncoder();
      await vscode.workspace.fs.writeFile(
        instructionsFile,
        encoder.encode(fileContent),
      );
      vscode.window.showInformationMessage(
        "Successfully created copilot-instructions.md",
      );
    }
  } catch (error) {
    log.error("Error updating copilot instructions file:", error);
    vscode.window.showErrorMessage(
      `Error updating copilot-instructions.md: ${error}`,
    );
  }
}

export function registerGhCopilotInstructionsCommand(
  context: vscode.ExtensionContext,
) {
  // Register command to update copilot instructions file
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.updateCopilotInstructions",
      updateGhCopilotInstructionsCommand,
    ),
  );
}
