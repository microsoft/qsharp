// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { EventType, sendTelemetryEvent, UserFlowStatus } from "./telemetry";

const codingInstructionsTitle = "# Q# coding instructions (updated April 2025)";

const codingInstructionsFooter = `---End: Q# coding instructions---\n\n`;

const codingInstructionsContent = `${codingInstructionsTitle}

Follow these instructions when generating Q# code in .qs files,
and Q# project folders that include a \`qsharp.json\` file.

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

A Q# project can reference a library from GitHub by updating the \`dependencies\` entry of
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
and maybe \`qsharp_widgets\` for visuals.

## Response formatting

Avoid using LaTeX in your responses to the user.

${codingInstructionsFooter}`;

/**
 * Checks if the copilot-instructions.md file exists and contains Q# instructions.
 *
 * @param workspaceFolder The workspace folder URI.
 * @returns A promise that resolves to true if the file exists and contains Q# instructions.
 */
async function hasQSharpCopilotInstructions(
  workspaceFolder: vscode.Uri,
): Promise<boolean> {
  try {
    const githubDir = vscode.Uri.joinPath(workspaceFolder, ".github");
    const instructionsFile = vscode.Uri.joinPath(
      githubDir,
      "copilot-instructions.md",
    );

    // Check if file exists. This will throw if the file doesn't exist.
    await vscode.workspace.fs.stat(instructionsFile);

    // Check if file contains Q# instructions
    const existingContent =
      await vscode.workspace.fs.readFile(instructionsFile);
    const existingText = new TextDecoder("utf-8").decode(existingContent);

    return existingText.includes(codingInstructionsTitle);
  } catch {
    // If any error occurs (file not found, etc.), return false
    return false;
  }
}

/**
 * Command to update or create the Copilot instructions file for Q#.
 * Shows a prompt to the user and updates the file if confirmed.
 */
async function updateGhCopilotInstructionsCommand() {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    vscode.window.showErrorMessage("No workspace folder is open");
    return;
  }

  for (const folder of workspaceFolders) {
    // Check if the file already exists with Q# instructions
    const hasInstructions = await hasQSharpCopilotInstructions(folder.uri);
    if (hasInstructions) {
      vscode.window.showInformationMessage(
        "copilot-instructions.md already contains Q# instructions",
      );
      return;
    }
  }

  // TODO: choose a workspace folder more intelligently

  return await updateCopilotInstructions(workspaceFolders[0].uri);
}

export async function updateCopilotInstructions(workspaceFolder: vscode.Uri) {
  if (await hasQSharpCopilotInstructions(workspaceFolder)) {
    // If the file already exists and contains Q# instructions, do nothing
    return;
  }

  sendTelemetryEvent(EventType.UpdateCopilotInstructionsStart, {}, {});

  // Show a yes/no prompt to the user
  const response = await vscode.window.showInformationMessage(
    "We're about to update your `copilot-instructions.md` file.\n\n" +
      "This file helps GitHub Copilot understand and work better with Q# files and features provided by the Quantum Development Kit extension.\n\n" +
      "Would you like to proceed with updating `copilot-instructions.md`?",
    { modal: true },
    { title: "Yes" },
    { title: "No", isCloseAffordance: true },
  );

  if (response?.title !== "Yes") {
    sendTelemetryEvent(EventType.UpdateCopilotInstructionsEnd, {
      reason: "User canceled",
      flowStatus: UserFlowStatus.Aborted,
    });
    return; // User canceled or dismissed the dialog
  }

  try {
    const githubDir = vscode.Uri.joinPath(workspaceFolder, ".github");
    const instructionsFile = vscode.Uri.joinPath(
      githubDir,
      "copilot-instructions.md",
    );

    // Create .github directory if doesn't exist
    await vscode.workspace.fs.createDirectory(githubDir);

    // Check if the file already exists
    try {
      const existingContent =
        await vscode.workspace.fs.readFile(instructionsFile);
      const existingText = new TextDecoder("utf-8").decode(existingContent);

      // We've confirmed above that the file doesn't already contain Q# instructions,
      // so append them.
      const updatedContent = existingText + "\n\n" + codingInstructionsContent;
      await vscode.workspace.fs.writeFile(
        instructionsFile,
        new TextEncoder().encode(updatedContent),
      );

      vscode.window.showInformationMessage(
        "Successfully updated copilot-instructions.md",
      );
    } catch {
      // File doesn't exist, create it
      await vscode.workspace.fs.writeFile(
        instructionsFile,
        new TextEncoder().encode(codingInstructionsContent),
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

    sendTelemetryEvent(
      EventType.UpdateCopilotInstructionsEnd,
      { flowStatus: UserFlowStatus.Failed, reason: "Error" },
      {},
    );
    return;
  }

  // Send telemetry event for successful completion
  sendTelemetryEvent(
    EventType.UpdateCopilotInstructionsEnd,
    { flowStatus: UserFlowStatus.Succeeded },
    {},
  );
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
