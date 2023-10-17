// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { IDebugServiceWorker, getDebugServiceWorker } from "qsharp-lang";
import { FileAccessor, qsharpExtensionId, isQsharpDocument } from "../common";
import { QscDebugSession } from "./session";

let debugServiceWorkerFactory: () => IDebugServiceWorker;

export async function activateDebugger(
  context: vscode.ExtensionContext,
): Promise<void> {
  const debugWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/debugger/debug-service-worker.js",
  );

  debugServiceWorkerFactory = () =>
    getDebugServiceWorker(
      debugWorkerScriptPath.toString(),
    ) as IDebugServiceWorker;
  registerCommands(context);

  const provider = new QsDebugConfigProvider();
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider("qsharp", provider),
  );

  const factory = new InlineDebugAdapterFactory();
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("qsharp", factory),
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
            { noDebug: true },
          );
        }
      },
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
            noDebug: false,
          });
        }
      },
    ),
  );
}

class QsDebugConfigProvider implements vscode.DebugConfigurationProvider {
  resolveDebugConfigurationWithSubstitutedVariables(
    folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration,
    _token?: vscode.CancellationToken | undefined,
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    // if launch.json is missing or empty
    if (!config.type && !config.request && !config.name) {
      const editor = vscode.window.activeTextEditor;
      if (editor && isQsharpDocument(editor.document)) {
        config.type = "qsharp";
        config.name = "Launch";
        config.request = "launch";
        config.program = editor.document.uri.toString();
        config.shots = 1;
        config.noDebug = "noDebug" in config ? config.noDebug : false;
        config.stopOnEntry = !config.noDebug;
      }
    } else {
      // we have a launch config, resolve the program path

      // ensure we have the program uri correctly formatted
      // this is a user specified path.
      if (config.program) {
        const uri = workspaceFileAccessor.resolvePathToUri(config.program);
        config.program = uri.toString();
      } else {
        // Use the active editor if no program or ${file} is specified.
        const editor = vscode.window.activeTextEditor;
        if (editor && isQsharpDocument(editor.document)) {
          config.program = editor.document.uri.toString();
        }
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

  resolveDebugConfiguration(
    folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration,
    _token?: vscode.CancellationToken | undefined,
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    // apply defaults if not set
    config.type = config.type ?? "qsharp";
    config.name = config.name ?? "Launch";
    config.request = config.request ?? "launch";
    config.shots = config.shots ?? 1;
    config.entry = config.entry ?? "";
    config.trace = config.trace ?? false;
    // noDebug is set to true when the user runs the program without debugging.
    // otherwise it usually isn't set, but we default to false.
    config.noDebug = config.noDebug ?? false;
    // stopOnEntry is set to true when the user runs the program with debugging.
    // unless overridden.
    config.stopOnEntry = config.stopOnEntry ?? !config.noDebug;

    return config;
  }
}

// The path normalization, fallbacks, and uri resolution are necessary
// due to https://github.com/microsoft/vscode-debugadapter-node/issues/298
// We can't specify that the debug adapter should use Uri for paths and can't
// use the DebugSession conversion functions because they don't work in the web.
export const workspaceFileAccessor: FileAccessor = {
  normalizePath(path: string): string {
    return path.replace(/\\/g, "/");
  },
  convertToWindowsPathSeparator(path: string): string {
    return path.replace(/\//g, "\\");
  },
  resolvePathToUri(path: string): vscode.Uri {
    const normalizedPath = this.normalizePath(path);
    return vscode.Uri.parse(normalizedPath, false);
  },
  async openPath(path: string): Promise<vscode.TextDocument> {
    const uri: vscode.Uri = this.resolvePathToUri(path);
    return this.openUri(uri);
  },
  async openUri(uri: vscode.Uri): Promise<vscode.TextDocument> {
    try {
      return await vscode.workspace.openTextDocument(uri);
    } catch {
      const path = this.convertToWindowsPathSeparator(uri.toString());
      return await vscode.workspace.openTextDocument(vscode.Uri.file(path));
    }
  },
};

class InlineDebugAdapterFactory
  implements vscode.DebugAdapterDescriptorFactory
{
  createDebugAdapterDescriptor(
    session: vscode.DebugSession,
    _executable: vscode.DebugAdapterExecutable | undefined,
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const worker = debugServiceWorkerFactory();
    const qscSession = new QscDebugSession(
      workspaceFileAccessor,
      worker,
      session.configuration,
    );
    return qscSession.init().then(() => {
      return new vscode.DebugAdapterInlineImplementation(qscSession);
    });
  }
}
