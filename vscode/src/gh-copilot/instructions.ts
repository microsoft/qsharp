// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { EventType, sendTelemetryEvent, UserFlowStatus } from "../telemetry";

/**
 * Command to update or create the Copilot instructions file for Q#.
 * Shows a prompt to the user and updates the file if confirmed.
 */
export async function updateCopilotInstructions(
  trigger: "Command" | "Project" | "Activation" | "ChatToolCall",
  context: vscode.ExtensionContext,
): Promise<string | undefined> {
  const globalStateUri = context.globalStorageUri;
  const userInvoked = trigger === "Command";

  if (isExtensionInstructionsConfigured(globalStateUri)) {
    if (userInvoked) {
      // fire-and-forget
      showInfoMessage("Copilot instructions for Q# are already configured.", {
        showSettingButton: true,
        learnMoreButton: true,
      });
    }
    return;
  }

  sendTelemetryEvent(
    EventType.UpdateCopilotInstructionsStart,
    {
      trigger,
    },
    {},
  );

  const response = await showConfirmationPrompt(userInvoked);

  if (response !== "Yes") {
    sendTelemetryEvent(EventType.UpdateCopilotInstructionsEnd, {
      reason: "user declined",
      flowStatus: UserFlowStatus.Aborted,
    });

    // fire-and-forget
    showInfoMessage(
      "To add Copilot instructions for Q# to your workspace at any time, " +
        'run the command "QDK: Update Copilot instructions file for Q#".',
      { showSettingButton: false },
    );

    return response; // User dismissed the dialog
  }

  try {
    await addExtensionInstructionsToUserConfig(globalStateUri);
    const removedOldInstructions = await removeOldQSharpCopilotInstructions();

    // fire-and-forget
    showInfoMessage(
      "Successfully configured Copilot instructions for Q#" +
        (removedOldInstructions
          ? ", and removed old Q# instructions from copilot-instructions.md."
          : "."),
      {
        showSettingButton: true,
        learnMoreButton: true,
      },
    );
  } catch (error) {
    log.error(`Error updating Copilot instructions`, error);
    vscode.window.showErrorMessage(
      `Could not update Copilot instructions for Q#.`,
    );

    sendTelemetryEvent(
      EventType.UpdateCopilotInstructionsEnd,
      { flowStatus: UserFlowStatus.Failed, reason: "Error" },
      {},
    );

    return response;
  }

  sendTelemetryEvent(
    EventType.UpdateCopilotInstructionsEnd,
    { flowStatus: UserFlowStatus.Succeeded },
    {},
  );
}

/**
 * Checks the user's instructionsFilesLocations setting to see if
 * our extension's instructions directory is already included.
 */
function isExtensionInstructionsConfigured(
  globalStateUri: vscode.Uri,
): boolean {
  const extensionInstructionsDir = getExtensionInstructionsDir(globalStateUri);
  const instructionsLocations = getConfiguredInstructionsFilesLocations();

  // Check if our directory is in the map as a key and it's enabled (true)
  if (instructionsLocations[extensionInstructionsDir] === true) {
    return true;
  }
  return false;
}

/**
 * Updates the user's instructionsFilesLocations setting to include
 * our extension's instructions directory.
 */
async function addExtensionInstructionsToUserConfig(
  globalStateUri: vscode.Uri,
): Promise<void> {
  const instructionsLocations = getConfiguredInstructionsFilesLocations();
  const extensionInstructionsDir = getExtensionInstructionsDir(globalStateUri);

  // Only add the extension's chat-instructions directory
  // if it's not already configured or if it's disabled
  if (instructionsLocations[extensionInstructionsDir] !== true) {
    // Create a new map with our directory set to true
    const updatedLocations = { ...instructionsLocations };
    updatedLocations[extensionInstructionsDir] = true;

    const config = vscode.workspace.getConfiguration("chat");
    await config.update(
      "instructionsFilesLocations",
      updatedLocations,
      vscode.ConfigurationTarget.Global,
    );
  }
}

/**
 * @returns the user's `chat.instructionsFilesLocations` setting.
 */
function getConfiguredInstructionsFilesLocations(): Record<string, boolean> {
  const config = vscode.workspace.getConfiguration("chat");
  const setting = config.get<Record<string, boolean>>(
    "instructionsFilesLocations",
    {},
  );
  return setting;
}

/**
 * Gets our extension's chat instructions directory's absolute path.
 * Will only work in *real* fileSystems -
 * TBD how this setting will work if/when VS Code supports GitHub Copilot in the browser.
 *
 * TODO: create GitHub issue to track how we handle this in the browser.
 */
function getExtensionInstructionsDir(globalStateUri: vscode.Uri): string {
  const instructionsUri = vscode.Uri.joinPath(
    globalStateUri,
    "chat-instructions",
  );

  // Normalize path by removing trailing slashes and replacing backslashes with forward slashes
  return instructionsUri.fsPath.replace(/[/\\]$/, "").replace(/\\/g, "/");
}

async function showConfirmationPrompt(userInvoked: boolean) {
  const buttons = [{ title: "Yes" }, { title: "No", isCloseAffordance: true }];

  let message =
    "Add Copilot instructions for Q#?\n\n" +
    "This will configure GitHub Copilot to work better with Q# and other Quantum Development Kit features.";

  let response: vscode.MessageItem | undefined;

  if (!userInvoked) {
    buttons.push({ title: "Don't show again" });
    // For non-modal dialogs, include a markdown link in the message
    message +=
      "\n\nLearn more at [https://aka.ms/qdk.copilot](https://aka.ms/qdk.copilot)";
    response = await vscode.window.showInformationMessage(message, ...buttons);
  } else {
    // For modal dialogs, add a Learn More button
    const allButtons = [...buttons, { title: "Learn More" }];

    response = await vscode.window.showInformationMessage(
      message,
      { modal: true },
      ...allButtons,
    );

    // Handle the "Learn More" button click
    if (response?.title === "Learn More") {
      vscode.env.openExternal(vscode.Uri.parse("https://aka.ms/qdk.copilot"));
      // Show the dialog again since clicking Learn More shouldn't dismiss it
      return await showConfirmationPrompt(userInvoked);
    }
  }

  return response?.title;
}

async function showInfoMessage(
  message: string,
  options: {
    showSettingButton?: boolean;
    learnMoreButton?: boolean;
  },
) {
  const buttons: string[] = [];
  if (options.showSettingButton) {
    buttons.push("Show Setting");
  }
  if (options.learnMoreButton) {
    buttons.push("Learn More");
  }
  const selection = await vscode.window.showInformationMessage(
    message,
    ...buttons,
  );
  if (selection === "Show Setting") {
    // Open the settings UI at our specific setting
    vscode.commands.executeCommand(
      "workbench.action.openSettings",
      "chat.instructionsFilesLocations",
    );
  } else if (selection === "Learn More") {
    // Open the documentation URL
    vscode.env.openExternal(vscode.Uri.parse("https://aka.ms/qdk.copilot"));
  }
}

/**
 * Removes old Q# instructions from the copilot-instructions.md file if they exist.
 * These were only added by the QDK extension in the April 2025 release.
 *
 * @returns true if instructions were found and removed, false otherwise.
 */
async function removeOldQSharpCopilotInstructions(): Promise<boolean> {
  const oldCodingInstructionsTitle =
    "# Q# coding instructions (updated April 2025)";
  const oldCodingInstructionsFooter = `<!-- End: Q# coding instructions -->\n\n`;

  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    return false;
  }

  let removed = false;

  for (const workspaceFolder of workspaceFolders) {
    const instructionsFile = getOldInstructionsFileLocation(
      workspaceFolder.uri,
    );

    let text = "";
    try {
      const content = await vscode.workspace.fs.readFile(instructionsFile);
      text = new TextDecoder("utf-8").decode(content);
      const startIndex = text.indexOf(oldCodingInstructionsTitle);
      if (startIndex === -1) {
        continue;
      }
      let endIndex = text.indexOf(oldCodingInstructionsFooter, startIndex);

      if (endIndex !== -1) {
        endIndex += oldCodingInstructionsFooter.length;
        // Skip any trailing newlines after the footer
        while (
          endIndex < text.length &&
          (text[endIndex] === "\n" || text[endIndex] === "\r")
        ) {
          endIndex++;
        }

        // Create new content without the Q# instructions
        const newContent =
          text.substring(0, startIndex) + text.substring(endIndex);

        // Write back the file without the Q# instructions
        await vscode.workspace.fs.writeFile(
          instructionsFile,
          new TextEncoder().encode(newContent),
        );
      }
      removed = true;
    } catch {
      // file doesn't exist or we couldn't edit it
      continue;
    }
  }

  return removed;
}

function getOldInstructionsFileLocation(
  workspaceFolder: vscode.Uri,
): vscode.Uri {
  return vscode.Uri.joinPath(
    workspaceFolder,
    ".github",
    "copilot-instructions.md",
  );
}

async function copyInstructionsFileToGlobalStorage(
  context: vscode.ExtensionContext,
) {
  const source = vscode.Uri.joinPath(
    context.extensionUri,
    "chat-instructions",
    "qsharp.instructions.md",
  );

  const target = vscode.Uri.joinPath(
    context.globalStorageUri,
    "chat-instructions",
    "qsharp.instructions.md",
  );

  try {
    await vscode.workspace.fs.copy(source, target, { overwrite: true });
    return true;
  } catch {
    log.warn(
      `Error copying Q# instructions file from ${source.toString()} to ${target.toString()}`,
    );
    return false;
  }
}

/**
 * Registers the command to configure GitHub Copilot to use Q# coding instructions.
 * This updates the chat.instructionsFilesLocations setting to include the extension's
 * chat-instructions directory, rather than creating a file in the user's workspace.
 */
export async function registerGhCopilotInstructionsCommand(
  context: vscode.ExtensionContext,
) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.updateCopilotInstructions",
      () => updateCopilotInstructions("Command", context),
    ),
  );

  // Copy the instructions file to the global storage location
  // The global storage location is stable across versions,
  // but our instructions content may change from version to version.
  await copyInstructionsFileToGlobalStorage(context);

  // Also do a one-time prompt at activation time
  if (
    context.globalState.get<boolean>(
      "showUpdateCopilotInstructionsPromptAtStartup",
      true,
    )
  ) {
    updateCopilotInstructions("Activation", context).then((response) => {
      if (response === "Don't show again") {
        context.globalState.update(
          "showUpdateCopilotInstructionsPromptAtStartup",
          false,
        );
      }
    });
  }
}
