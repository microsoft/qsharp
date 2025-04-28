// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { assert } from "chai";
import * as vscode from "vscode";
import { activateExtension, withMockedInfoMessage } from "../extensionUtils";

suite("Q# Copilot Instructions Tests", function suite() {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");

  const workspaceFolderUri = workspaceFolder.uri;
  const githubDirUri = vscode.Uri.joinPath(workspaceFolderUri, ".github");
  const copilotInstructionsUri = vscode.Uri.joinPath(
    githubDirUri,
    "copilot-instructions.md",
  );

  // Cleanup function to delete test files
  async function cleanupTestFiles() {
    try {
      // Delete the test file if it exists
      await vscode.workspace.fs.delete(copilotInstructionsUri, {
        useTrash: false,
      });
      await vscode.workspace.fs.delete(githubDirUri, { useTrash: false });
    } catch {
      // Ignore error if file doesn't exist
    }
  }

  this.beforeAll(async () => {
    await activateExtension();
  });

  this.beforeEach(async () => {
    await cleanupTestFiles();
  });

  test("Command creates new copilot instructions file", async function () {
    // Make sure test files don't exist before starting
    await cleanupTestFiles();

    // Use the helper function to mock the information message with "Yes" response
    await withMockedInfoMessage("Yes", async () => {
      // Execute the command
      await vscode.commands.executeCommand(
        "qsharp-vscode.updateCopilotInstructions",
      );

      // Add a small delay to ensure file operations complete
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Verify the .github directory was created
      const githubDirStats = await vscode.workspace.fs.stat(githubDirUri);
      assert.equal(
        githubDirStats.type,
        vscode.FileType.Directory,
        ".github directory should be created",
      );

      // Verify the copilot-instructions.md file was created
      const fileStats = await vscode.workspace.fs.stat(copilotInstructionsUri);
      assert.equal(
        fileStats.type,
        vscode.FileType.File,
        "copilot-instructions.md file should be created",
      );

      // Verify the file content
      const fileContent = await vscode.workspace.fs.readFile(
        copilotInstructionsUri,
      );
      const content = new TextDecoder().decode(fileContent);

      assert.include(
        content,
        "# Q# coding instructions",
        "File should contain the correct header",
      );
      assert.include(
        content,
        "# Q# coding instructions",
        "File should contain placeholder text",
      );
    });
  });

  test("Command appends to existing file", async function () {
    this.timeout(10000); // Increase timeout for this test

    // Create a test file first with different content
    const testContent =
      "# Existing instructions\n\nDo not remove this content.\n";
    const encoder = new TextEncoder();

    // Make sure the .github directory exists
    try {
      await vscode.workspace.fs.stat(githubDirUri);
    } catch {
      await vscode.workspace.fs.createDirectory(githubDirUri);
    }

    // Create the file with test content
    await vscode.workspace.fs.writeFile(
      copilotInstructionsUri,
      encoder.encode(testContent),
    );

    // Use the helper function to mock the information message with "Yes" response
    await withMockedInfoMessage("Yes", async () => {
      // Execute the command
      await vscode.commands.executeCommand(
        "qsharp-vscode.updateCopilotInstructions",
      );

      // Add a small delay to ensure file operations complete
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Verify the file was updated (not replaced)
      const fileContent = await vscode.workspace.fs.readFile(
        copilotInstructionsUri,
      );
      const content = new TextDecoder().decode(fileContent);

      assert.include(
        content,
        "# Existing instructions",
        "File should keep existing content",
      );
      assert.include(
        content,
        "Do not remove this content",
        "File should keep existing content",
      );
      assert.include(
        content,
        "# Q# coding instructions",
        "File should append Q# instructions header",
      );
    });
  });

  test("Command does nothing when user selects 'No'", async function () {
    this.timeout(10000); // Increase timeout for this test

    // Create a test file first with known content
    const testContent = "# Test content that should not change\n";
    const encoder = new TextEncoder();

    // Ensure .github directory exists
    try {
      await vscode.workspace.fs.stat(githubDirUri);
    } catch {
      await vscode.workspace.fs.createDirectory(githubDirUri);
    }

    // Create the file with test content
    await vscode.workspace.fs.writeFile(
      copilotInstructionsUri,
      encoder.encode(testContent),
    );

    // Use the helper function to mock the information message with "No" response
    await withMockedInfoMessage("No", async () => {
      // Execute the command
      await vscode.commands.executeCommand(
        "qsharp-vscode.updateCopilotInstructions",
      );

      // Add a small delay to ensure file operations complete
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Verify the file was not changed
      const fileContent = await vscode.workspace.fs.readFile(
        copilotInstructionsUri,
      );
      const content = new TextDecoder().decode(fileContent);

      assert.equal(
        content,
        testContent,
        "File content should not change when user selects 'No'",
      );
      assert.notInclude(
        content,
        "# Q# coding instructions",
        "File should not contain new instructions",
      );
    });
  });
});
