/* eslint-disable @typescript-eslint/no-unused-vars */
import * as vscode from "vscode";
import {
  log,
  getCompiler,
  ICompilerWorker,
  getCompilerWorker,
  ICompiler,
  QscEventTarget,
} from "qsharp";

import {
  InitializedEvent,
  DebugSession,
  ExitedEvent,
  TerminatedEvent,
} from "@vscode/debugadapter";
import { DebugProtocol } from "@vscode/debugprotocol";

// Don't seem to be able to create a new Worker. Just use a singleton compile for now.
let simulator: ICompiler;

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

export async function registerDebugger(
  context: vscode.ExtensionContext,
  compiler: ICompiler
) {
  log.info("Registering the qsharp debugger");
  simulator = compiler;
  const provider = new QscDebugConfigProvider();

  //   workerScriptPath = vscode.Uri.joinPath(
  //     context.extensionUri,
  //     "./out/simulatorWorker.js"
  //   );

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

    //const worker = new Worker(workerScriptPath.toString());

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

    const document = vscode.workspace.textDocuments.filter(
      (doc) => doc.uri.toString() === this.programBeingDebugged?.toString()
    )[0];

    const source = document.getText();
    const eventTarget = new QscEventTarget(false);
    const debugConsole = vscode.debug.activeDebugConsole;

    eventTarget.addEventListener("Message", (evt) => {
      debugConsole.appendLine(`Message: ${evt.detail}`);
    });

    eventTarget.addEventListener("DumpMachine", (evt) => {
      function formatComplex(real: number, imag: number) {
        // Format -0 as 0
        // Also using Unicode Minus Sign instead of ASCII Hyphen-Minus
        // and Unicode Mathematical Italic Small I instead of ASCII i.
        const r = `${real <= -0.00005 ? "−" : ""}${Math.abs(real).toFixed(4)}`;
        const i = `${imag <= -0.00005 ? "−" : "+"}${Math.abs(imag).toFixed(
          4
        )}𝑖`;
        return `${r}${i}`;
      }

      function probability(real: number, imag: number) {
        return real * real + imag * imag;
      }

      const dump = evt.detail;
      debugConsole.appendLine("\nDumpMachine:\n");
      debugConsole.appendLine(
        "  Basis | Amplitude     | Probability   | Phase"
      );
      debugConsole.appendLine(
        "  ---------------------------------------------"
      );
      Object.keys(dump).map((basis) => {
        const [real, imag] = dump[basis];
        const complex = formatComplex(real, imag);
        const probabilityPercent = probability(real, imag) * 100;
        const phase = Math.atan2(imag, real);

        debugConsole.appendLine(
          `  ${basis}  | ${complex} | ${probabilityPercent.toFixed(
            4
          )}%     | ${phase.toFixed(4)}`
        );
      });
      debugConsole.appendLine("\n");
    });

    eventTarget.addEventListener("Result", (evt) => {
      const resultJson = JSON.stringify(evt.detail.value, null, 2);
      debugConsole.appendLine(`Result: ${resultJson}`);
    });

    debugConsole.appendLine("Q# program running in simulator...\n");

    simulator.run(source, "", 1, eventTarget);

    debugConsole.appendLine("\nQ# simulation completed.");
    this.sendEvent(new TerminatedEvent());
    this.sendEvent(new ExitedEvent(0));
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
