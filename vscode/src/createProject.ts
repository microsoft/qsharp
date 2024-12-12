// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log, samples } from "qsharp-lang";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { qsharpExtensionId } from "./common";

export async function initProjectCreator(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.createProject`,
      async (folderUri: vscode.Uri | undefined) => {
        sendTelemetryEvent(EventType.CreateProject, {}, {});

        if (!folderUri) {
          // This was run from the command palette, not the context menu, so create the project
          // in the root of an open workspace folder.
          const workspaceCount = vscode.workspace.workspaceFolders?.length || 0;
          if (workspaceCount === 0) {
            // No workspaces open
            vscode.window.showErrorMessage(
              "You must have a folder open to create a Q# project in",
            );
            return;
          } else if (workspaceCount === 1) {
            folderUri = vscode.workspace.workspaceFolders![0].uri;
          } else {
            const workspaceChoice = await vscode.window.showWorkspaceFolderPick(
              { placeHolder: "Pick the workspace folder for the project" },
            );
            if (!workspaceChoice) return;
            folderUri = workspaceChoice.uri;
          }
        }

        const edit = new vscode.WorkspaceEdit();
        const projUri = vscode.Uri.joinPath(folderUri, "qsharp.json");
        const mainUri = vscode.Uri.joinPath(folderUri, "src", "Main.qs");

        const sample = samples.find((elem) => elem.title === "Minimal");
        if (!sample) {
          // Should never happen.
          log.error("Unable to find the Minimal sample");
          return;
        }

        edit.createFile(projUri);
        edit.createFile(mainUri);

        edit.set(projUri, [
          new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), "{}"),
        ]);
        edit.set(mainUri, [
          new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), sample.code),
        ]);

        // This doesn't throw on failure, it just returns false
        if (!(await vscode.workspace.applyEdit(edit))) {
          vscode.window.showErrorMessage(
            "Unable to create the project. Check the project files don't already exist and that the file system is writable",
          );
        }
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.populateFilesList`,
      async (qsharpJsonUri: vscode.Uri | undefined) => {
        // If called from the content menu qsharpJsonUri will be the full qsharp.json uri
        // If called from the command palette is will be undefined, so use the active editor
        log.info("populateQsharpFilesList called with", qsharpJsonUri);

        qsharpJsonUri =
          qsharpJsonUri ?? vscode.window.activeTextEditor?.document.uri;
        if (!qsharpJsonUri) {
          log.error(
            "populateFilesList called, but argument or active editor is not qsharp.json",
          );
          return;
        }

        log.debug("Populating qsharp.json files for: ", qsharpJsonUri.path);

        // First, verify the qsharp.json can be opened and is a valid json file
        const qsharpJsonDoc =
          await vscode.workspace.openTextDocument(qsharpJsonUri);
        if (!qsharpJsonDoc) {
          log.error("Unable to open the qsharp.json file at ", qsharpJsonDoc);
          return;
        }

        let manifestObj: any = {};
        try {
          manifestObj = JSON.parse(qsharpJsonDoc.getText());
        } catch {
          await vscode.window.showErrorMessage(
            `Unable to parse the contents of ${qsharpJsonUri.path}`,
          );
          return;
        }

        // Recursively find all .qs documents under the ./src dir
        const files: string[] = [];
        const srcDir = vscode.Uri.joinPath(qsharpJsonUri, "..", "src");

        // Verify the src directory exists
        try {
          const srcDirStat = await vscode.workspace.fs.stat(srcDir);
          if (srcDirStat.type !== vscode.FileType.Directory) {
            await vscode.window.showErrorMessage(
              "The ./src path is not a directory",
            );
            return;
          }
        } catch {
          await vscode.window.showErrorMessage(
            "The ./src directory does not exist. Create the directory and add .qs files to it",
          );
          return;
        }

        async function getQsFilesInDir(dir: vscode.Uri) {
          const dirFiles = (await vscode.workspace.fs.readDirectory(dir)).sort(
            (a, b) => {
              // To order the list, put files before directories, then sort alphabetically
              if (a[1] !== b[1]) return a[1] < b[1] ? -1 : 1;
              return a[0] < b[0] ? -1 : 1;
            },
          );
          for (const [name, type] of dirFiles) {
            if (type === vscode.FileType.File && name.endsWith(".qs")) {
              files.push(vscode.Uri.joinPath(dir, name).toString());
            } else if (type === vscode.FileType.Directory) {
              await getQsFilesInDir(vscode.Uri.joinPath(dir, name));
            }
          }
          return files;
        }
        await getQsFilesInDir(srcDir);

        // Update the files property of the qsharp.json and write back to the document
        const srcDirPrefix = srcDir.toString() + "";
        manifestObj["files"] = files.map((file) =>
          file.replace(srcDirPrefix, "src"),
        );

        // Apply the edits to the qsharp.json
        const edit = new vscode.WorkspaceEdit();
        edit.replace(
          qsharpJsonUri,
          new vscode.Range(0, 0, qsharpJsonDoc.lineCount, 0),
          JSON.stringify(manifestObj, null, 2),
        );
        if (!(await vscode.workspace.applyEdit(edit))) {
          vscode.window.showErrorMessage(
            "Unable to update the qsharp.json file. Check the file is writable",
          );
          return;
        }

        // Bring the qsharp.json to the front for the user to save
        await vscode.window.showTextDocument(qsharpJsonDoc);
      },
    ),
  );

  type LocalProjectRef = {
    path: string; // Absolute or relative path to the project dir
  };

  type GitHubProjectRef = {
    github: {
      owner: string;
      repo: string;
      ref: string;
      path?: string; // Optional, defaults to the root of the repo
    };
  };

  type Dependency = LocalProjectRef | GitHubProjectRef;

  // TODO: Replace with a list of legitimate known Q# projects on GitHub
  const githubProjects: { [name: string]: GitHubProjectRef } = {
    // Add a template to the end of the list users can use to easily add their own
    "<id>": {
      github: {
        owner: "<owner>",
        repo: "<project>",
        ref: "<commit>",
      },
    },
  };

  // Given two directory paths, return the relative path from the first to the second
  function getRelativeDirPath(from: string, to: string): string {
    // Ensure we have something
    if (!from || !to) throw "Invalid arguments";

    // Trim trailing slashes (even from the root "/" case)
    if (from.endsWith("/")) from = from.slice(0, -1);
    if (to.endsWith("/")) to = to.slice(0, -1);

    // Break both paths into their components
    const fromParts = from.split("/");
    const toParts = to.split("/");

    // Remove the common beginning of the paths
    while (fromParts[0] === toParts[0]) {
      fromParts.shift();
      toParts.shift();
    }

    // Add a .. for each remaining part in the from path
    let result = "";
    while (fromParts.length) {
      result += "../";
      fromParts.shift();
    }
    // Add the remaining path from the to path
    result += toParts.join("/");
    if (result.endsWith("/")) {
      result = result.slice(0, -1);
    }
    return result;
  }

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.addProjectReference`,
      async (qsharpJsonUri: vscode.Uri | undefined) => {
        // If called from the content menu qsharpJsonUri will be the full qsharp.json uri
        // If called from the command palette is will be undefined, so use the active editor
        log.info("addProjectReference called with", qsharpJsonUri);

        qsharpJsonUri =
          qsharpJsonUri ?? vscode.window.activeTextEditor?.document.uri;
        if (!qsharpJsonUri) {
          log.error(
            "addProjectReference called, but argument or active editor is not qsharp.json",
          );
          return;
        }

        log.debug("Adding project reference to ", qsharpJsonUri.path);

        // First, verify the qsharp.json can be opened and is a valid json file
        const qsharpJsonDoc =
          await vscode.workspace.openTextDocument(qsharpJsonUri);
        if (!qsharpJsonDoc) {
          log.error("Unable to open the qsharp.json file at ", qsharpJsonDoc);
          return;
        }
        const qsharpJsonDir = vscode.Uri.joinPath(qsharpJsonUri, "..");

        let manifestObj: any = {};
        try {
          manifestObj = JSON.parse(qsharpJsonDoc.getText());
        } catch {
          await vscode.window.showErrorMessage(
            `Unable to parse the contents of ${qsharpJsonUri.path}`,
          );
          return;
        }

        // Find all the other Q# projects in the workspace
        const projectFiles = (
          await vscode.workspace.findFiles("**/qsharp.json")
        ).filter((file) => file.toString() !== qsharpJsonUri.toString());

        const projectChoices: Array<{ name: string; ref: Dependency }> = [];

        projectFiles.forEach((file) => {
          const dirName = file.path.slice(0, -"/qsharp.json".length);
          const relPath = getRelativeDirPath(qsharpJsonDir.path, dirName);
          projectChoices.push({
            name: dirName.slice(dirName.lastIndexOf("/") + 1),
            ref: {
              path: relPath,
            },
          });
        });

        Object.keys(githubProjects).forEach((name) => {
          projectChoices.push({
            name: name,
            ref: githubProjects[name],
          });
        });

        // Convert any spaces, dashes, dots, tildes, or quotes in project names
        // to underscores. (Leave more 'exotic' non-identifier patterns to the user to fix)
        //
        // Note: At some point we may want to detect/avoid duplicate names, e.g. if the user already
        // references a project via 'foo', and they add a reference to a 'foo' on GitHub or in another dir.
        projectChoices.forEach(
          (val, idx, arr) =>
            (arr[idx].name = val.name.replace(/[- "'.~]/g, "_")),
        );

        const folderIcon = new vscode.ThemeIcon("folder");
        const githubIcon = new vscode.ThemeIcon("github");

        // Ask the user to pick a project to add as a reference
        const projectChoice = await vscode.window.showQuickPick(
          projectChoices.map((choice) => {
            if ("github" in choice.ref) {
              return {
                label: choice.name,
                detail: `github://${choice.ref.github.owner}/${choice.ref.github.repo}#${choice.ref.github.ref}`,
                iconPath: githubIcon,
                ref: choice.ref,
              };
            } else {
              return {
                label: choice.name,
                detail: choice.ref.path,
                iconPath: folderIcon,
                ref: choice.ref,
              };
            }
          }),
          { placeHolder: "Pick a project to add as a reference" },
        );

        if (!projectChoice) {
          log.info("User cancelled project choice");
          return;
        }

        log.info("User picked project: ", projectChoice);

        if (!manifestObj["dependencies"]) manifestObj["dependencies"] = {};
        manifestObj["dependencies"][projectChoice.label] = projectChoice.ref;

        // Apply the edits to the qsharp.json
        const edit = new vscode.WorkspaceEdit();
        edit.replace(
          qsharpJsonUri,
          new vscode.Range(0, 0, qsharpJsonDoc.lineCount, 0),
          JSON.stringify(manifestObj, null, 2),
        );
        if (!(await vscode.workspace.applyEdit(edit))) {
          vscode.window.showErrorMessage(
            "Unable to update the qsharp.json file. Check the file is writable",
          );
          return;
        }

        // Bring the qsharp.json to the front for the user to save
        await vscode.window.showTextDocument(qsharpJsonDoc);
      },
    ),
  );
}
