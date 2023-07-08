/* eslint-disable @typescript-eslint/no-unused-vars */
import * as vscode from "vscode";
import { log } from "qsharp";

import {
  InitializedEvent,
  DebugSession,
  Source,
  ExitedEvent,
  TerminatedEvent,
} from "@vscode/debugadapter";
import { DebugProtocol } from "@vscode/debugprotocol";

class InlineDebugAdapterFactory
  implements vscode.DebugAdapterDescriptorFactory
{
  createDebugAdapterDescriptor(): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    return new vscode.DebugAdapterInlineImplementation(new QscDebugSession());
  }
}

class QscDebugConfigProvider implements vscode.DebugConfigurationProvider {
  resolveDebugConfiguration(
    _folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    // if launch.json is missing or empty
    log.info("In resolveDebugConfiguration");
    if (!config.type && !config.request && !config.name) {
      const editor = vscode.window.activeTextEditor;
      if (editor && editor.document.languageId === "qsharp") {
        config.type = "qsharp";
        config.name = "Launch";
        config.request = "launch";
        config.program = editor.document.uri.toString();
        config.stopOnEntry = false;
      }
      log.debug(`Set launch config for file: ${config.program}`);
    }

    if (!config.program) {
      return vscode.window
        .showInformationMessage("Cannot find a program to debug")
        .then(() => {
          return undefined; // abort launch
        });
    }

    return config;
  }
}

export async function registerDebugger(context: vscode.ExtensionContext) {
  log.info("Registering the qsharp debugger");
  const provider = new QscDebugConfigProvider();

  context.subscriptions.push(
    vscode.commands.registerTextEditorCommand(
      "extension.qsharp-debug.runEditorContents",
      (textEditor: vscode.TextEditor) => {
        if (textEditor.document.languageId !== "qsharp") return;

        const targetResource = textEditor.document.uri;
        log.debug("Setting targetResource to " + targetResource.toString());
        if (targetResource) {
          vscode.debug.startDebugging(
            undefined,
            {
              type: "qsharp",
              name: "Run File",
              request: "launch",
              stopOnEntry: false,
              program: targetResource.toString(),
            },
            { noDebug: true }
          );
        }
      }
    )
  );

  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider("qsharp", provider)
  );

  const factory = new InlineDebugAdapterFactory();
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("qsharp", factory)
  );
}

// See https://code.visualstudio.com/updates/v1_42#_implement-a-debug-adapter-inside-an-extension
export class QscDebugSession extends DebugSession {
  programBeingDebugged?: vscode.Uri;

  constructor() {
    super();
  }

  // TODO
  protected initializeRequest(
    response: DebugProtocol.InitializeResponse,
    _args: DebugProtocol.InitializeRequestArguments
  ): void {
    log.debug("In QsxDebugSession::initializeRequest");

    response.body = response.body || {};
    response.body.supportsDisassembleRequest = false;
    response.body.supportsSteppingGranularity = false;
    response.body.supportsInstructionBreakpoints = false;
    response.body.supportsSetVariable = false;
    response.body.supportsFunctionBreakpoints = false;
    response.body.supportsStepBack = false;
    response.body.supportsBreakpointLocationsRequest = false;

    this.sendResponse(response);
    this.sendEvent(new InitializedEvent());
  }
  protected launchRequest(
    response: DebugProtocol.LaunchResponse,
    args: DebugProtocol.LaunchRequestArguments,
    request?: DebugProtocol.Request | undefined
  ): void {
    log.debug("In QsxDebugSession::launchRequest");
    this.programBeingDebugged = vscode.Uri.parse((args as any).program);

    log.debug("Launch called for: " + this.programBeingDebugged.toString());

    this.sendResponse(response);

    // TODO: Run the simulator here, report progress to Terminal, then exit.
    vscode.debug.activeDebugConsole.appendLine(
      "Q# program running in simulator"
    );
    setTimeout(() => {
      vscode.debug.activeDebugConsole.appendLine("Q# simulation completed");
      log.debug("Sending exit event after launch request");
      this.sendEvent(new TerminatedEvent());
      this.sendEvent(new ExitedEvent(0));
    }, 2000);
  }

  protected threadsRequest(
    response: DebugProtocol.ThreadsResponse,
    request?: DebugProtocol.Request | undefined
  ): void {
    log.debug("In QsxDebugSession::threadsRequest");
    response.body = { threads: [{ id: 1, name: "main" }] };
    this.sendResponse(response);
  }
}
