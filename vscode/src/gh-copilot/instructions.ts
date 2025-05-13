// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { EventType, sendTelemetryEvent, UserFlowStatus } from "../telemetry";
import { getActiveQSharpDocumentUri } from "../programConfig";

const codingInstructionsTitle = "# Q# coding instructions (updated April 2025)";

const codingInstructionsFooter = `<!-- End: Q# coding instructions -->\n\n`;

const codingInstructionsContent = `${codingInstructionsTitle}

Follow these instructions when generating Q# code in .qs files,
and Q# project folders that include a \`qsharp.json\` file.

## Syntax

When writing Q#, the \`for (i in 1..100)\` syntax is invalid, use \`for i in 1..100\` or
\`for element in array[2...]\`.

While many Q# operators are C-like, it uses \`or\` instead of \`||\` and \`and\` instead of \`&&\`.

To extract values from a tuple, use destructuring via the \`let (item0, item1) = tupleValue;\` syntax.

## Project structure

### Single-file projects

Q# files don't always need to exist in a project. A single \`.qs\` file can be compiled and
run without a \`qsharp.json\` file. Prefer a single \`.qs\` file for simple programs.

### Multi-file projects

When Q# source files need to to reference each other, a \`qsharp.json\` file must be
created. Source files must exist under the \`src\` folder.

Example layout:

\`\`\`
project_root
|--qsharp.json
|--src
|--|--Main.qs
|--|--Tests.qs
\`\`\`

A typical \`qsharp.json\` will be a JSON file with an empty JSON object in it.

\`\`\`json
{}
\`\`\`

Modern Q# does not use \`namespace\` blocks to enclose code.
Each function or operation is in a namespace which is the name of the containing file.
For example, if \`Main.qs\` has an operation \`Foo\`, then \`Tests.qs\` could reference the
operation as \`Main.Foo\`, or bring \`Foo\` into scope by adding \`import Main.Foo;\` in the file.

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

## Setup and tools

The Quantum Development Kit (QDK) was re-written at the start of 2024 and no longer uses
the IQ# Jupyter kernel, or the \`dotnet\` command line tools. Job management is best handled
now via tool calls integration into GitHub Copilot, or via Python code using the \`qsharp\`
and \`azure-quantum\` packages.

To execute Q# code, use the provided tools.

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
async function updateGhCopilotInstructionsCommand(userInvoked: boolean) {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    vscode.window.showErrorMessage("No workspace folder is open");
    return;
  }

  let resourceUri;
  if (workspaceFolders.length === 1) {
    // Single workspace folder, just use that
    resourceUri = workspaceFolders[0].uri;
  } else {
    // Prefer the workspace of the active Q# document if available
    const currentDoc = getActiveQSharpDocumentUri();
    resourceUri = currentDoc ?? workspaceFolders[0].uri;
  }

  return await updateCopilotInstructions(resourceUri, userInvoked);
}

export async function updateCopilotInstructions(
  resource: vscode.Uri,
  userInvoked: boolean,
): Promise<vscode.MessageItem | undefined> {
  // Always add copilot instructions in the workspace root
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(resource)?.uri;
  if (!workspaceFolder) {
    return;
  }

  if (await hasQSharpCopilotInstructions(workspaceFolder)) {
    // If the file already exists and contains Q# instructions, do nothing
    return;
  }

  sendTelemetryEvent(
    EventType.UpdateCopilotInstructionsStart,
    {
      trigger: userInvoked ? "user" : "startup",
    },
    {},
  );

  const buttons = [{ title: "Yes" }, { title: "No", isCloseAffordance: true }];
  if (!userInvoked) {
    buttons.push({ title: "Don't show again" });
  }

  const modal = userInvoked;

  const response = await vscode.window.showInformationMessage(
    "Add Q# guidance to copilot-instructions.md?\n\n" +
      "Updating this file will help GitHub Copilot understand and work better with Q# files and other Quantum Development Kit features.\n\n" +
      "Learn more at " +
      (modal
        ? "https://aka.ms/qdk.copilot" // links don't render in modal dialogs
        : "[https://aka.ms/qdk.copilot](https://aka.ms/qdk.copilot)"),
    {
      modal,
    },
    ...buttons,
  );

  if (response?.title !== "Yes") {
    sendTelemetryEvent(EventType.UpdateCopilotInstructionsEnd, {
      reason: "User canceled",
      flowStatus: UserFlowStatus.Aborted,
    });

    vscode.window.showInformationMessage(
      "To add Q# guidance to copilot-instructions.md at any time, " +
        'run the command "Q#: Update Copilot instructions file for Q#".',
    );

    return response; // User dismissed the dialog
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

    return response;
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
      () => updateGhCopilotInstructionsCommand(true),
    ),
  );

  // Also do a one-time prompt at startup
  if (
    context.globalState.get<boolean>(
      "showUpdateCopilotInstructionsPromptAtStartup",
      true,
    )
  ) {
    updateGhCopilotInstructionsCommand(false).then((response) => {
      if (response?.title === "Don't show again") {
        context.globalState.update(
          "showUpdateCopilotInstructionsPromptAtStartup",
          false,
        );
      }
    });
  }
}

// some test code to try out configuration updates
// log.info("QSharpTools initialized");
// const cfg = vscode.workspace.getConfiguration("chat");
// const val = cfg.inspect("agent")?.globalValue;
// cfg.update("agent.enabled", true, ConfigurationTarget.Global).then(
//   () => {
//     const lastVal = cfg.inspect("agent")?.globalValue;
//     log.info("Agent config value: ", lastVal);
//   },
//   (e) => {
//     log.error("Failed to update agent config", e);
//   },
// );
// log.info("Agent config value: ", val);
