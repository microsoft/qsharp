// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";

import {
  ExitedEvent,
  InitializedEvent,
  Logger,
  LoggingDebugSession,
  TerminatedEvent,
  logger,
  Breakpoint,
  StoppedEvent,
  Thread,
  StackFrame,
  Source,
  OutputEvent,
  Handles,
  Scope,
} from "@vscode/debugadapter";

import { FileAccessor, basename } from "../common";
import { DebugProtocol } from "@vscode/debugprotocol";
import {
  IBreakpointSpan,
  IDebugServiceWorker,
  log,
  StepResultId,
  IStructStepResult,
  QscEventTarget,
  qsharpLibraryUriScheme,
} from "qsharp";
import { createDebugConsoleEventTarget } from "./output";
import { ILaunchRequestArguments } from "./types";
const ErrorProgramHasErrors = "program contains compile errors(s): cannot run.";
const SimulationCompleted = "Q# simulation completed.";
const ConfigurationDelayMS = 1000;

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class QscDebugSession extends LoggingDebugSession {
  private static threadID = 1;

  private breakpointLocations: Map<string, IBreakpointSpan[]>;
  private breakpoints: Map<string, DebugProtocol.Breakpoint[]>;
  private variableHandles = new Handles<"locals" | "quantum">();
  private failed: boolean;
  private program: vscode.Uri;
  private eventTarget: QscEventTarget;
  private supportsVariableType = false;

  public constructor(
    private fileAccessor: FileAccessor,
    private debugService: IDebugServiceWorker,
    private config: vscode.DebugConfiguration
  ) {
    super();

    this.program = fileAccessor.resolvePathToUri(this.config.program);
    this.failed = false;
    this.eventTarget = createDebugConsoleEventTarget((message) => {
      this.writeToStdOut(message);
    });

    this.breakpointLocations = new Map<string, IBreakpointSpan[]>();
    this.breakpoints = new Map<string, DebugProtocol.Breakpoint[]>();
    this.setDebuggerLinesStartAt1(false);
    this.setDebuggerColumnsStartAt1(false);
  }

  public async init(): Promise<void> {
    const programText = (
      await this.fileAccessor.openUri(this.program)
    ).getText();

    const loaded = await this.debugService.loadSource(
      this.program.toString(),
      programText
    );
    if (loaded) {
      const locations = await this.debugService.getBreakpoints(
        this.program.toString()
      );
      log.trace(`init breakpointLocations: %O`, locations);
      this.breakpointLocations.set(this.program.toString(), locations);
    } else {
      log.warn(`compilation failed.`);
      this.failed = true;
    }
  }

  /**
   * The 'initialize' request is the first request called by the frontend
   * to interrogate the features the debug adapter provides.
   */
  protected initializeRequest(
    response: DebugProtocol.InitializeResponse,
    args: DebugProtocol.InitializeRequestArguments
  ): void {
    this.supportsVariableType = args.supportsVariableType ?? false;

    // build and return the capabilities of this debug adapter:
    response.body = response.body || {};

    // the adapter implements the configurationDone request.
    response.body.supportsConfigurationDoneRequest = true;

    // make VS Code show a 'step back' button
    response.body.supportsStepBack = false;

    // make VS Code support data breakpoints
    response.body.supportsDataBreakpoints = false;

    // make VS Code support completion in REPL
    response.body.supportsCompletionsRequest = false;

    // the adapter defines two exceptions filters, one with support for conditions.
    response.body.supportsExceptionFilterOptions = false;

    // make VS Code send exceptionInfo request
    response.body.supportsExceptionInfoRequest = false;

    // make VS Code able to read and write variable memory
    response.body.supportsReadMemoryRequest = false;
    response.body.supportsWriteMemoryRequest = false;

    response.body.supportSuspendDebuggee = false;
    response.body.supportTerminateDebuggee = true;
    response.body.supportsFunctionBreakpoints = true;
    response.body.supportsRestartRequest = false;

    // make VS Code send the breakpointLocations request
    response.body.supportsBreakpointLocationsRequest = true;

    /* Settings that we need to eventually support: */

    // make VS Code send cancel request
    response.body.supportsCancelRequest = false;

    // make VS Code use 'evaluate' when hovering over source
    response.body.supportsEvaluateForHovers = false;

    response.body.supportsDelayedStackTraceLoading = false;

    // make VS Code provide "Step in Target" functionality
    response.body.supportsStepInTargetsRequest = false;

    // make VS Code send setVariable request
    response.body.supportsSetVariable = false;

    // make VS Code send setExpression request
    response.body.supportsSetExpression = false;

    // make VS Code send disassemble request
    response.body.supportsDisassembleRequest = false;
    response.body.supportsSteppingGranularity = false;

    response.body.supportsInstructionBreakpoints = false;

    this.sendResponse(response);

    // since this debug adapter can accept configuration requests like 'setBreakpoint' at any time,
    // we request them early by sending an 'initializeRequest' to the frontend.
    // The frontend will end the configuration sequence by calling 'configurationDone' request.
    this.sendEvent(new InitializedEvent());
  }

  /**
   * Called at the end of the configuration sequence.
   * Indicates that all breakpoints etc. have been sent to the DA and that the 'launch' can start.
   */
  protected configurationDoneRequest(
    response: DebugProtocol.ConfigurationDoneResponse,
    args: DebugProtocol.ConfigurationDoneArguments
  ): void {
    super.configurationDoneRequest(response, args);

    // notify the launchRequest that configuration has finished
    this.emit("sessionConfigurationDone");
  }

  protected async launchRequest(
    response: DebugProtocol.LaunchResponse,
    args: ILaunchRequestArguments
  ): Promise<void> {
    if (this.failed) {
      log.info(
        "compilation failed. sending error response and stopping execution."
      );
      this.sendErrorResponse(response, {
        id: -1,
        format: ErrorProgramHasErrors,
        showUser: true,
      });
      return;
    }

    // configure DAP logging
    logger.setup(
      args.trace ? Logger.LogLevel.Verbose : Logger.LogLevel.Stop,
      false
    );

    // wait until configuration has finished (configurationDoneRequest has been called)
    const configurationDone: Promise<void> = new Promise((resolve, reject) => {
      this.once("sessionConfigurationDone", resolve);
    });
    await Promise.race([configurationDone, delay(ConfigurationDelayMS)]);

    // This needs to be done before we start executing below
    // in order to ensure that the eventTarget is ready to receive
    // events from the debug service. Otherwise, we may miss events
    // that are sent before the active debug session is set.
    log.trace(`sending launchRequest response`);
    this.sendResponse(response);

    if (args.noDebug) {
      log.trace(`Running without debugging`);
      await this.runWithoutDebugging(args);
    } else {
      log.trace(`Running with debugging`);
      await this.continue();
    }
  }

  private async eval_step(step: () => Promise<IStructStepResult>) {
    await step().then(
      async (result) => {
        if (result.id == StepResultId.BreakpointHit) {
          const evt = new StoppedEvent(
            "breakpoint",
            QscDebugSession.threadID
          ) as DebugProtocol.StoppedEvent;
          evt.body.hitBreakpointIds = [result.value];
          log.trace(`raising breakpoint event`);
          this.sendEvent(evt);
        } else if (result.id == StepResultId.Return) {
          await this.endSession(`ending session`, 0);
        } else {
          log.trace(`step result: ${result.id} ${result.value}`);
          this.sendEvent(new StoppedEvent("step", QscDebugSession.threadID));
        }
      },
      (error) => {
        this.endSession(`ending session due to error: ${error}`, 1);
      }
    );
  }

  private async continue(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalContinue(bps, this.eventTarget)
    );
  }

  private async next(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalNext(bps, this.eventTarget)
    );
  }

  private async stepIn(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalStepIn(bps, this.eventTarget)
    );
  }

  private async stepOut(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalStepOut(bps, this.eventTarget)
    );
  }

  private async endSession(message: string, exitCode: number): Promise<void> {
    log.trace(message);
    this.writeToDebugConsole("");
    this.writeToDebugConsole(SimulationCompleted);
    this.sendEvent(new TerminatedEvent());
    this.sendEvent(new ExitedEvent(exitCode));
  }

  private async runWithoutDebugging(
    args: ILaunchRequestArguments
  ): Promise<void> {
    const bps: number[] = [];
    for (let i = 0; i < args.shots; i++) {
      await this.eval_step(
        async () => await this.debugService.evalContinue(bps, this.eventTarget)
      );
    }
  }

  private getBreakpointIds(): number[] {
    const bps: number[] = [];
    for (const bp of this.breakpoints.get(this.program.toString()) ?? []) {
      if (bp && bp.id) {
        bps.push(bp.id);
      }
    }

    return bps;
  }

  protected async continueRequest(
    response: DebugProtocol.ContinueResponse,
    args: DebugProtocol.ContinueArguments
  ): Promise<void> {
    log.trace(`continueRequest: %O`, args);

    log.trace(`sending continue response`);
    this.sendResponse(response);

    await this.continue();
  }

  protected async nextRequest(
    response: DebugProtocol.NextResponse,
    args: DebugProtocol.NextArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`nextRequest: %O`, args);

    this.sendResponse(response);
    await this.next();
  }

  protected async stepInRequest(
    response: DebugProtocol.StepInResponse,
    args: DebugProtocol.StepInArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`stepInRequest: %O`, args);
    this.sendResponse(response);

    await this.stepIn();
  }

  protected async stepOutRequest(
    response: DebugProtocol.StepOutResponse,
    args: DebugProtocol.StepOutArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`stepOutRequest: %O`, args);
    this.sendResponse(response);

    await this.stepOut();
  }

  protected async breakpointLocationsRequest(
    response: DebugProtocol.BreakpointLocationsResponse,
    args: DebugProtocol.BreakpointLocationsArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`breakpointLocationsRequest: %O`, args);

    response.body = {
      breakpoints: [],
    };

    const file = await this.fileAccessor
      .openPath(args.source.path ?? "")
      .catch((e) => {
        log.error(`Failed to open file: ${e}`);
        const fileUri = this.fileAccessor.resolvePathToUri(
          args.source.path ?? ""
        );
        log.trace(
          "breakpointLocationsRequest, target file: " + fileUri.toString()
        );
      });

    const targetLineNumber = this.convertClientLineToDebugger(args.line);
    if (file && targetLineNumber < file.lineCount) {
      // Map request start/end line/column to file offset for debugger
      const line = file.lineAt(targetLineNumber);
      const lineRange = line.range;
      const startLine = lineRange.start.line;
      const startCol = args.column
        ? this.convertClientColumnToDebugger(args.column)
        : lineRange.start.character;
      const endLine = args.endLine
        ? this.convertClientLineToDebugger(args.endLine)
        : lineRange.end.line;
      const endCol = args.endColumn
        ? this.convertClientColumnToDebugger(args.endColumn)
        : lineRange.end.character;
      const startPos = new vscode.Position(startLine, startCol);
      const endPos = new vscode.Position(endLine, endCol);
      const startOffset = file.offsetAt(startPos);
      const endOffset = file.offsetAt(endPos);

      log.trace(
        `breakpointLocationsRequest: ${startLine}:${startCol} - ${endLine}:${endCol}`
      );
      log.trace(`breakpointLocationsRequest: ${startOffset} - ${endOffset}`);
      // Now that we have the mapped breakpoint span, get the actual breakpoints
      // from the debugger
      // This currently has issues with breakpoints that span multiple lines
      // stmt for example may have a span that only includes the identifier
      // where the rest of the statement is on the next line(s)
      const bps =
        this.breakpointLocations
          .get(file.uri.toString())
          ?.filter((bp) => startOffset <= bp.lo && bp.hi <= endOffset) ?? [];

      log.trace(`breakpointLocationsRequest: candidates %O`, bps);

      // must map the debugger breakpoints back to the client breakpoint locations
      const bls = bps.map((bps) => {
        const startPos = file.positionAt(bps.lo);
        const endPos = file.positionAt(bps.hi);
        const bp: DebugProtocol.BreakpointLocation = {
          line: startPos.line,
          column: this.convertDebuggerColumnToClient(startPos.character),
          endLine: endPos.line,
          endColumn: this.convertDebuggerColumnToClient(endPos.character),
        };
        return bp;
      });
      log.trace(`breakpointLocationsRequest: mapped %O`, bls);
      response.body = {
        breakpoints: bls,
      };
    }
    log.trace(`breakpointLocationsResponse: %O`, response);
    this.sendResponse(response);
  }

  protected async setBreakPointsRequest(
    response: DebugProtocol.SetBreakpointsResponse,
    args: DebugProtocol.SetBreakpointsArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`setBreakPointsRequest: %O`, args);

    const file = await this.fileAccessor
      .openPath(args.source.path ?? "")
      .catch((e) => {
        log.error(`setBreakPointsRequest - Failed to open file: ${e}`);
        const fileUri = this.fileAccessor.resolvePathToUri(
          args.source.path ?? ""
        );
        log.trace("setBreakPointsRequest, target file: " + fileUri.toString());
      });

    if (file) {
      log.trace(`setBreakPointsRequest: looking`);
      this.breakpoints.set(file.uri.toString(), []);
      log.trace(
        `setBreakPointsRequest: files in cache %O`,
        this.breakpointLocations.keys()
      );
      const locations = this.breakpointLocations.get(file.uri.toString()) ?? [];
      log.trace(`setBreakPointsRequest: got locations %O`, locations);
      // convert the request line/column to file offset for debugger
      const bpOffsets: [lo: number, hi: number][] = (args.breakpoints ?? [])
        .filter(
          (sourceBreakpoint) =>
            this.convertClientLineToDebugger(sourceBreakpoint.line) <
            file.lineCount
        )
        .map((sourceBreakpoint) => {
          const line = this.convertClientLineToDebugger(sourceBreakpoint.line);
          const lineRange = file.lineAt(line).range;
          const startCol = sourceBreakpoint.column
            ? this.convertClientColumnToDebugger(sourceBreakpoint.column)
            : lineRange.start.character;
          const startPos = new vscode.Position(line, startCol);
          const startOffset = file.offsetAt(startPos);
          const endOffset = file.offsetAt(lineRange.end);

          return [startOffset, endOffset];
        });

      // We should probably ensure we don't return duplicate
      // spans from the debugger, but for now we'll just filter them out
      const uniqOffsets: [lo: number, hi: number][] = [];
      for (const bpOffset of bpOffsets) {
        if (
          uniqOffsets.findIndex(
            (u) => u[0] == bpOffset[0] && u[1] == bpOffset[1]
          ) == -1
        ) {
          uniqOffsets.push(bpOffset);
        }
      }
      // Now that we have the mapped breakpoint span, get the actual breakpoints
      // with corresponding ids from the debugger
      const bps = [];
      for (const bpOffset of uniqOffsets) {
        for (const location of locations) {
          // Check if the location is within the breakpoint span
          // The span from the API is wider than the HIR as it includes
          // the entire line
          if (bpOffset[0] <= location.lo && location.hi <= bpOffset[1]) {
            const bp = this.createBreakpoint(
              location.id,
              file.positionAt(location.lo),
              file.positionAt(location.hi)
            );

            bps.push(bp);
          }
        }
      }

      // Update our breakpoint list for the given file
      this.breakpoints.set(file.uri.toString(), bps);

      response.body = {
        breakpoints: bps,
      };
    }
    log.trace(`setBreakPointsResponse: %O`, response);
    this.sendResponse(response);
  }

  protected threadsRequest(response: DebugProtocol.ThreadsResponse): void {
    log.trace(`threadRequest`);
    response.body = {
      threads: [new Thread(QscDebugSession.threadID, "thread 1")],
    };
    log.trace(`threadResponse: %O`, response);
    this.sendResponse(response);
  }

  protected async stackTraceRequest(
    response: DebugProtocol.StackTraceResponse,
    args: DebugProtocol.StackTraceArguments
  ): Promise<void> {
    log.trace(`stackTraceRequest: %O`, args);
    const debuggerStackFrames = await this.debugService.getStackFrames();
    log.trace(`frames: %O`, debuggerStackFrames);
    const filterUndefined = <V>(value: V | undefined): value is V =>
      value != null;
    const mappedStackFrames = await Promise.all(
      debuggerStackFrames
        .map(async (f, id) => {
          log.trace(`frames: path %O`, f.path);

          const file = await this.fileAccessor
            .openPath(f.path ?? "")
            .catch((e) => {
              log.error(`stackTraceRequest - Failed to open file: ${e}`);
              const fileUri = this.fileAccessor.resolvePathToUri(f.path ?? "");
              log.trace(
                "stackTraceRequest, target file: " + fileUri.toString()
              );
            });
          if (file) {
            log.trace(`frames: file %O`, file);
            const start_pos = file.positionAt(f.lo);
            const end_pos = file.positionAt(f.hi);
            const sf: DebugProtocol.StackFrame = new StackFrame(
              id,
              f.name,
              new Source(
                basename(f.path) ?? f.path,
                file.uri.toString(true),
                undefined,
                undefined,
                "qsharp-adapter-data"
              ),
              this.convertDebuggerLineToClient(start_pos.line),
              this.convertDebuggerColumnToClient(start_pos.character)
            );
            sf.endLine = this.convertDebuggerLineToClient(end_pos.line);
            sf.endColumn = this.convertDebuggerColumnToClient(
              end_pos.character
            );
            return sf;
          } else {
            try {
              // This file isn't part of the workspace, so we'll
              // create a URI which can try to load it from the core and std lib
              // There is a custom content provider subscribed to this scheme.
              // Opening the text document by that uri will use the content
              // provider to look for the source code.
              const uri = vscode.Uri.from({
                scheme: qsharpLibraryUriScheme,
                path: f.path,
              });
              const file = await this.fileAccessor.openUri(uri);
              const start_pos = file.positionAt(f.lo);
              const end_pos = file.positionAt(f.hi);
              const source = new Source(
                basename(f.path) ?? f.path,
                uri.toString(),
                0,
                "internal core/std library",
                "qsharp-adapter-data"
              ) as DebugProtocol.Source;
              const sf = new StackFrame(
                id,
                f.name,
                source as Source,
                this.convertDebuggerLineToClient(start_pos.line),
                this.convertDebuggerColumnToClient(start_pos.character)
              );
              sf.endLine = this.convertDebuggerLineToClient(end_pos.line);
              sf.endColumn = this.convertDebuggerColumnToClient(
                end_pos.character
              );

              return sf as DebugProtocol.StackFrame;
            } catch (e: any) {
              log.warn(e.message);
              return new StackFrame(
                id,
                f.name,
                undefined,
                undefined,
                undefined
              );
            }
          }
        })
        .filter(filterUndefined)
    );
    const stackFrames = mappedStackFrames.reverse();
    stackFrames.push(
      new StackFrame(0, "entry", undefined) as DebugProtocol.StackFrame
    );
    response.body = {
      stackFrames: stackFrames,
      totalFrames: stackFrames.length,
    };

    log.trace(`stackTraceResponse: %O`, response);
    this.sendResponse(response);
  }

  protected disconnectRequest(
    response: DebugProtocol.DisconnectResponse,
    args: DebugProtocol.DisconnectArguments,
    request?: DebugProtocol.Request
  ): void {
    log.trace(`disconnectRequest: %O`, args);
    this.debugService.terminate();
    this.sendResponse(response);
    log.trace(`disconnectResponse: %O`, response);
  }

  protected scopesRequest(
    response: DebugProtocol.ScopesResponse,
    args: DebugProtocol.ScopesArguments
  ): void {
    log.trace(`scopesRequest: %O`, args);
    response.body = {
      scopes: [
        new Scope(
          "Quantum State",
          this.variableHandles.create("quantum"),
          true
        ),
        new Scope("Locals", this.variableHandles.create("locals"), false),
      ],
    };
    log.trace(`scopesResponse: %O`, response);
    this.sendResponse(response);
  }

  protected async variablesRequest(
    response: DebugProtocol.VariablesResponse,
    args: DebugProtocol.VariablesArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`variablesRequest: ${JSON.stringify(args, null, 2)}`);

    response.body = {
      variables: [],
    };

    const handle = this.variableHandles.get(args.variablesReference);
    if (handle === "locals") {
      const locals = await this.debugService.getLocalVariables();
      const variables = locals.map((local) => {
        const variable: DebugProtocol.Variable = {
          name: local.name,
          value: local.value,
          variablesReference: 0,
        };
        if (this.supportsVariableType) {
          variable.type = local.var_type;
        }
        return variable;
      });
      response.body = {
        variables: variables,
      };
    } else if (handle === "quantum") {
      const state = await this.debugService.captureQuantumState();
      const variables: DebugProtocol.Variable[] = state.map((entry) => {
        const variable: DebugProtocol.Variable = {
          name: entry.name,
          value: entry.value,
          variablesReference: 0,
          type: "Complex",
        };
        return variable;
      });
      response.body = {
        variables: variables,
      };
    }

    log.trace(`variablesResponse: %O`, response);
    this.sendResponse(response);
  }

  private createBreakpoint(
    id: number,
    startPos: vscode.Position,
    endPos: vscode.Position
  ): DebugProtocol.Breakpoint {
    const verified = true;
    const bp = new Breakpoint(verified) as DebugProtocol.Breakpoint;
    bp.id = id;
    bp.line = this.convertDebuggerLineToClient(startPos.line);
    bp.column = this.convertDebuggerColumnToClient(startPos.character);
    bp.endLine = this.convertDebuggerLineToClient(endPos.line);
    bp.endColumn = this.convertDebuggerColumnToClient(endPos.character);
    return bp;
  }

  private writeToStdOut(message: string): void {
    const evt: DebugProtocol.OutputEvent = new OutputEvent(
      `${message}\n`,
      "stdout"
    );
    this.sendEvent(evt);
  }

  private writeToDebugConsole(message: string): void {
    const evt: DebugProtocol.OutputEvent = new OutputEvent(
      `${message}\n`,
      "console"
    );
    this.sendEvent(evt);
  }
}
