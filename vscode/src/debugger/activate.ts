// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { IDebugServiceWorker, getDebugServiceWorker } from "qsharp";
import { FileAccessor, qsharpExtensionId } from "../common";
import { QscDebugSession } from "./session";

let debugServiceWorkerFactory: () => IDebugServiceWorker;

export async function activateDebugger(
  context: vscode.ExtensionContext
): Promise<void> {
  const compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js"
  );
  const debugWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/debugger/debug-service-worker.js"
  );

  debugServiceWorkerFactory = () =>
    getDebugServiceWorker(
      debugWorkerScriptPath.toString()
    ) as IDebugServiceWorker;
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
              program: targetResource.toString(),
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
            program: targetResource.toString(),
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
        config.program = editor.document.uri.toString();
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
  async readFile(uri: string): Promise<Uint8Array> {
    return await vscode.workspace.fs.readFile(vscode.Uri.parse(uri));
  },
  async readFileAsString(uri: string): Promise<string> {
    const contents = await this.readFile(uri);
    return new TextDecoder().decode(contents);
  },
  async writeFile(uri: string, contents: Uint8Array) {
    await vscode.workspace.fs.writeFile(vscode.Uri.parse(uri), contents);
  },
};

class InlineDebugAdapterFactory
  implements vscode.DebugAdapterDescriptorFactory
{
  createDebugAdapterDescriptor(
    session: vscode.DebugSession,
    _executable: vscode.DebugAdapterExecutable | undefined
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const worker = debugServiceWorkerFactory();
    const qscSession = new QscDebugSession(
      workspaceFileAccessor,
      worker,
      session.configuration
    );
    return qscSession.init().then(() => {
      return new vscode.DebugAdapterInlineImplementation(qscSession);
    });
  }
}
