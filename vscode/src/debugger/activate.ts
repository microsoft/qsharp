// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { IDebugServiceWorker, getDebugServiceWorker } from "qsharp";
import {
  FileAccessor,
  qsharpExtensionId,
  qsharpDocumentFilter,
} from "../common";
import { QscDebugSession } from "./session";

let debugServiceWorkerFactory: () => IDebugServiceWorker;

export async function activateDebugger(
  context: vscode.ExtensionContext
): Promise<void> {
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
      if (
        editor &&
        vscode.languages.match(qsharpDocumentFilter, editor.document)
      ) {
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
  normalizePath(path: string): string {
    return path.replace(/\\/g, "/");
  },
  resolvePathToUri(path: string): vscode.Uri {
    const normalizedPath = this.normalizePath(path);
    return vscode.Uri.parse(normalizedPath, false);
  },
  async openFile(path: string): Promise<vscode.TextDocument> {
    const uri: vscode.Uri = this.resolvePathToUri(path);
    return await vscode.workspace.openTextDocument(uri);
  },
  async openUri(uri: vscode.Uri): Promise<vscode.TextDocument> {
    return await vscode.workspace.openTextDocument(uri);
  },
  async readFile(path: string): Promise<Uint8Array> {
    let uri: vscode.Uri = this.resolvePathToUri(path);
    return await vscode.workspace.fs.readFile(uri);
  },
  async readFileAsString(path: string): Promise<string> {
    const contents = await this.readFile(path);
    return new TextDecoder().decode(contents);
  },
  async writeFile(path: string, contents: Uint8Array) {
    await vscode.workspace.fs.writeFile(this.resolvePathToUri(path), contents);
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
