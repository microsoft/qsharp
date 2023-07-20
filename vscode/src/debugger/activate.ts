// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { ICompiler, getCompilerWorker } from "qsharp";
import { FileAccessor, qsharpExtensionId } from "../common";
import { QscDebugSession } from "./session";

let compiler: ICompiler;

export async function activateDebugger(
  context: vscode.ExtensionContext
): Promise<void> {
  const workerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js"
  );
  compiler = getCompilerWorker(workerScriptPath.toString()) as ICompiler;

  registerCommands(context);

  const provider = new QsDebugConfigProvider();
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider("qsharp", provider)
  );

  const factory = new InlineDebugAdapterFactory();
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("qsharp", factory)
  );
}

function registerCommands(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.runEditorContents`,
      (resource: vscode.Uri) => {
        let targetResource = resource;
        if (!targetResource && vscode.window.activeTextEditor) {
          targetResource = vscode.window.activeTextEditor.document.uri;
        }

        if (targetResource) {
          vscode.debug.startDebugging(
            undefined,
            {
              type: "qsharp",
              name: "Run Q# File",
              request: "launch",
              program: targetResource,
              shots: 1,
              stopOnEntry: false,
            },
            { noDebug: true }
          );
        }
      }
    ),
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.debugEditorContents`,
      (resource: vscode.Uri) => {
        let targetResource = resource;
        if (!targetResource && vscode.window.activeTextEditor) {
          targetResource = vscode.window.activeTextEditor.document.uri;
        }

        if (targetResource) {
          vscode.debug.startDebugging(undefined, {
            type: "qsharp",
            name: "Debug Q# File",
            request: "launch",
            program: targetResource,
            shots: 1,
            stopOnEntry: true,
          });
        }
      }
    )
  );
}

class QsDebugConfigProvider implements vscode.DebugConfigurationProvider {
  resolveDebugConfiguration(
    folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration,
    _token?: vscode.CancellationToken | undefined
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    // if launch.json is missing or empty
    if (!config.type && !config.request && !config.name) {
      const editor = vscode.window.activeTextEditor;
      if (editor && editor.document.languageId === "qsharp") {
        config.type = "qsharp";
        config.name = "Launch";
        config.request = "launch";
        config.program = editor.document.uri;
        config.shots = 1;
        config.stopOnEntry = true;
        config.noDebug = "noDebug" in config ? config.noDebug : false;
      }
    }

    if (!config.program) {
      // abort launch
      return vscode.window
        .showInformationMessage("Cannot find a Q# program to debug")
        .then((_) => {
          return undefined;
        });
    }

    return config;
  }
}

export const workspaceFileAccessor: FileAccessor = {
  async readFile(uri: vscode.Uri): Promise<Uint8Array> {
    return await vscode.workspace.fs.readFile(uri);
  },
  async readFileAsString(uri: vscode.Uri): Promise<string> {
    const contents = await this.readFile(uri);
    return new TextDecoder().decode(contents);
  },
  async writeFile(uri: vscode.Uri, contents: Uint8Array) {
    await vscode.workspace.fs.writeFile(uri, contents);
  },
};

class InlineDebugAdapterFactory
  implements vscode.DebugAdapterDescriptorFactory
{
  createDebugAdapterDescriptor(
    _session: vscode.DebugSession,
    _executable: vscode.DebugAdapterExecutable | undefined
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    return new vscode.DebugAdapterInlineImplementation(
      new QscDebugSession(workspaceFileAccessor, compiler)
    );
  }
}
