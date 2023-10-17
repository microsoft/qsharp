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

import { FileAccessor, basename, isQsharpDocument } from "../common";
import { DebugProtocol } from "@vscode/debugprotocol";
import {
  IBreakpointSpan,
  IDebugServiceWorker,
  log,
  StepResultId,
  IStructStepResult,
  QscEventTarget,
  qsharpLibraryUriScheme,
} from "qsharp-lang";
import { createDebugConsoleEventTarget } from "./output";
import { ILaunchRequestArguments } from "./types";
import { EventType, sendTelemetryEvent } from "../telemetry";
const ErrorProgramHasErrors =
  "program contains compile errors(s): cannot run. See debug console for more details.";
const SimulationCompleted = "Q# simulation completed.";
const ConfigurationDelayMS = 1000;

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Represents file local information about a breakpoint location. The values
// can't be calculated without a file, so they are stored here after
// initialization.
interface IFileBreakpointLocation {
  startOffset: number;
  endOffset: number;
  startPos: vscode.Position;
  endPos: vscode.Position;
  bpLocation: DebugProtocol.BreakpointLocation;
}

// All offsets are utf16 units/characters. The debugger offsets are utf8
// units/bytes, but they've been converted to utf16 units/characters so that any
// time we are in JS we can use consistent encoding offsets.
// The debugger location stores the offsets for the start & end of the span
// The file location stores the offsets as seen by the TextDocument for the file
// ui/breakpoint location stores the utf16 offsets as seen by vscode (line/col are 1 based)
interface IBreakpointLocationData {
  debuggerLocation: IBreakpointSpan;
  fileLocation: IFileBreakpointLocation;
  uiLocation: DebugProtocol.BreakpointLocation;
  breakpoint: DebugProtocol.Breakpoint;
}

export class QscDebugSession extends LoggingDebugSession {
  private static threadID = 1;

  private breakpointLocations: Map<string, IBreakpointLocationData[]>;
  private breakpoints: Map<string, DebugProtocol.Breakpoint[]>;
  private variableHandles = new Handles<"locals" | "quantum">();
  private failureMessage: string;
  private program: vscode.Uri;
  private eventTarget: QscEventTarget;
  private supportsVariableType = false;

  public constructor(
    private fileAccessor: FileAccessor,
    private debugService: IDebugServiceWorker,
    private config: vscode.DebugConfiguration,
  ) {
    super();

    this.program = fileAccessor.resolvePathToUri(this.config.program);
    this.failureMessage = "";
    this.eventTarget = createDebugConsoleEventTarget((message) => {
      this.writeToStdOut(message);
    });

    this.breakpointLocations = new Map<string, IBreakpointLocationData[]>();
    this.breakpoints = new Map<string, DebugProtocol.Breakpoint[]>();
    this.setDebuggerLinesStartAt1(false);
    this.setDebuggerColumnsStartAt1(false);
  }

  public async init(): Promise<void> {
    const start = performance.now();
    const file = await this.fileAccessor.openUri(this.program);
    const programText = file.getText();

    const failureMessage = await this.debugService.loadSource(
      this.program.toString(),
      programText,
      this.config.entry,
    );
    if (failureMessage == "") {
      const locations = await this.debugService.getBreakpoints(
        this.program.toString(),
      );
      log.trace(`init breakpointLocations: %O`, locations);
      const mapped = locations.map((location) => {
        const startPos = file.positionAt(location.lo);
        const endPos = file.positionAt(location.hi);
        const bpLocation: DebugProtocol.BreakpointLocation = {
          line: startPos.line,
          column: startPos.character,
          endLine: endPos.line,
          endColumn: endPos.character,
        };
        const fileLocation = {
          startOffset: file.offsetAt(startPos),
          endOffset: file.offsetAt(endPos),
          startPos: startPos,
          endPos: endPos,
          bpLocation: bpLocation,
        };
        const uiLocation: DebugProtocol.BreakpointLocation = {
          line: this.convertDebuggerLineToClient(startPos.line),
          column: this.convertDebuggerColumnToClient(startPos.character),
          endLine: this.convertDebuggerLineToClient(endPos.line),
          endColumn: this.convertDebuggerColumnToClient(endPos.character),
        };
        return {
          debuggerLocation: location,
          fileLocation: fileLocation,
          uiLocation: uiLocation,
          breakpoint: this.createBreakpoint(location.id, uiLocation),
        } as IBreakpointLocationData;
      });
      this.breakpointLocations.set(this.program.toString(), mapped);
    } else {
      log.warn(`compilation failed. ${failureMessage}`);
      this.failureMessage = failureMessage;
    }
    const end = performance.now();
    sendTelemetryEvent(
      EventType.DebugSessionStart,
      {},
      { timeToStartMs: end - start },
    );
  }

  /**
   * The 'initialize' request is the first request called by the frontend
   * to interrogate the features the debug adapter provides.
   */
  protected initializeRequest(
    response: DebugProtocol.InitializeResponse,
    args: DebugProtocol.InitializeRequestArguments,
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
    args: DebugProtocol.ConfigurationDoneArguments,
  ): void {
    super.configurationDoneRequest(response, args);

    // notify the launchRequest that configuration has finished
    this.emit("sessionConfigurationDone");
  }

  protected async launchRequest(
    response: DebugProtocol.LaunchResponse,
    args: ILaunchRequestArguments,
  ): Promise<void> {
    if (this.failureMessage != "") {
      log.info(
        "compilation failed. sending error response and stopping execution.",
      );
      this.writeToDebugConsole(this.failureMessage);
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
      false,
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
      if (this.config.stopOnEntry) {
        await this.stepIn();
      } else {
        await this.continue();
      }
    }
  }

  private async eval_step(step: () => Promise<IStructStepResult>) {
    await step().then(
      async (result) => {
        if (result.id == StepResultId.BreakpointHit) {
          const evt = new StoppedEvent(
            "breakpoint",
            QscDebugSession.threadID,
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
      },
    );
  }

  private async continue(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalContinue(bps, this.eventTarget),
    );
  }

  private async next(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalNext(bps, this.eventTarget),
    );
  }

  private async stepIn(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalStepIn(bps, this.eventTarget),
    );
  }

  private async stepOut(): Promise<void> {
    const bps = this.getBreakpointIds();
    await this.eval_step(
      async () => await this.debugService.evalStepOut(bps, this.eventTarget),
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
    args: ILaunchRequestArguments,
  ): Promise<void> {
    const bps: number[] = [];
    // This will be replaced when the interpreter
    // supports shots.
    for (let i = 0; i < args.shots; i++) {
      const result = await this.debugService.evalContinue(
        bps,
        this.eventTarget,
      );
      if (result.id != StepResultId.Return) {
        await this.endSession(`execution didn't run to completion`, -1);
        return;
      }
      this.writeToDebugConsole(`Finished shot ${i + 1} of ${args.shots}`);
      // Reset the interpreter for the next shot.
      // The interactive interpreter doesn't do this automatically,
      // and doesn't know how to deal with shots like the stateless version.
      await this.init();
      if (this.failureMessage != "") {
        log.info(
          "compilation failed. sending error response and stopping execution.",
        );
        this.writeToDebugConsole(this.failureMessage);
        await this.endSession(`ending session`, -1);
        return;
      }
    }
    await this.endSession(`ending session`, 0);
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
    args: DebugProtocol.ContinueArguments,
  ): Promise<void> {
    log.trace(`continueRequest: %O`, args);

    log.trace(`sending continue response`);
    this.sendResponse(response);

    await this.continue();
  }

  protected async nextRequest(
    response: DebugProtocol.NextResponse,
    args: DebugProtocol.NextArguments,
    request?: DebugProtocol.Request,
  ): Promise<void> {
    log.trace(`nextRequest: %O`, args);

    this.sendResponse(response);
    await this.next();
  }

  protected async stepInRequest(
    response: DebugProtocol.StepInResponse,
    args: DebugProtocol.StepInArguments,
    request?: DebugProtocol.Request,
  ): Promise<void> {
    log.trace(`stepInRequest: %O`, args);
    this.sendResponse(response);

    await this.stepIn();
  }

  protected async stepOutRequest(
    response: DebugProtocol.StepOutResponse,
    args: DebugProtocol.StepOutArguments,
    request?: DebugProtocol.Request,
  ): Promise<void> {
    log.trace(`stepOutRequest: %O`, args);
    this.sendResponse(response);

    await this.stepOut();
  }

  protected async breakpointLocationsRequest(
    response: DebugProtocol.BreakpointLocationsResponse,
    args: DebugProtocol.BreakpointLocationsArguments,
    request?: DebugProtocol.Request,
  ): Promise<void> {
    log.trace(`breakpointLocationsRequest: %O`, args);

    response.body = {
      breakpoints: [],
    };

    const file = await this.fileAccessor
      .openPath(args.source.path ?? "")
      .catch((e) => {
        log.trace(`Failed to open file: ${e}`);
        const fileUri = this.fileAccessor.resolvePathToUri(
          args.source.path ?? "",
        );
        log.trace(
          "breakpointLocationsRequest, target file: " + fileUri.toString(),
        );
        return undefined;
      });

    // If we couldn't find the file, or it wasn't a Q# file, or
    // the range is longer than the file, just return
    const targetLineNumber = this.convertClientLineToDebugger(args.line);
    if (
      !file ||
      !isQsharpDocument(file) ||
      targetLineNumber >= file.lineCount
    ) {
      log.trace(`setBreakPointsResponse: %O`, response);
      this.sendResponse(response);
      return;
    }

    // Map request start/end line/column to file offset for debugger
    // everything from `file` is 0 based, everything from `args` is 1 based
    // so we have to convert anything from `args` to 0 based

    const line = file.lineAt(targetLineNumber);
    const lineRange = line.range;
    // If the column isn't specified, it is a line breakpoint so that we
    // use the whole line's range for breakpoint finding.
    const isLineBreakpoint = !args.column;
    const startLine = lineRange.start.line;
    // If the column isn't specified, use the start of the line. This also means
    // that we are looking at the whole line for a breakpoint
    const startCol = args.column
      ? this.convertClientColumnToDebugger(args.column)
      : lineRange.start.character;
    // If the end line isn't specified, use the end of the line range
    const endLine = args.endLine
      ? this.convertClientLineToDebugger(args.endLine)
      : lineRange.end.line;
    // If the end column isn't specified, use the end of the line.
    const endCol = args.endColumn
      ? this.convertClientColumnToDebugger(args.endColumn)
      : lineRange.end.character;

    // We've translated the request's range into a full implied range,
    // calculate the start and end positions as offsets which can be used to
    // isolate statements.
    const startPos = new vscode.Position(startLine, startCol);
    const endPos = new vscode.Position(endLine, endCol);
    const startOffset = file.offsetAt(startPos);
    const endOffset = file.offsetAt(endPos);

    log.trace(
      `breakpointLocationsRequest: ${startLine}:${startCol} - ${endLine}:${endCol}`,
    );
    log.trace(`breakpointLocationsRequest: ${startOffset} - ${endOffset}`);

    // Now that we have the mapped breakpoint span, get the potential
    // breakpoints from the debugger

    // If is is a line breakpoint, we can just use the line number for matching
    // Otherwise, when looking for range breakpoints, we are given a single
    // column offset, so we need to check if the startOffset is within range.
    const bps =
      this.breakpointLocations
        .get(file.uri.toString())
        ?.filter((bp) =>
          isLineBreakpoint
            ? bp.uiLocation.line == args.line
            : startOffset <= bp.fileLocation.startOffset &&
              bp.fileLocation.startOffset <= endOffset,
        ) ?? [];

    log.trace(`breakpointLocationsRequest: candidates %O`, bps);

    // must map the debugger breakpoints back to the client breakpoint locations
    const bls = bps.map((bps) => {
      return bps.uiLocation;
    });
    log.trace(`breakpointLocationsRequest: mapped %O`, bls);
    response.body = {
      breakpoints: bls,
    };

    log.trace(`breakpointLocationsResponse: %O`, response);
    this.sendResponse(response);
  }

  protected async setBreakPointsRequest(
    response: DebugProtocol.SetBreakpointsResponse,
    args: DebugProtocol.SetBreakpointsArguments,
    request?: DebugProtocol.Request,
  ): Promise<void> {
    log.trace(`setBreakPointsRequest: %O`, args);

    const file = await this.fileAccessor
      .openPath(args.source.path ?? "")
      .catch((e) => {
        log.trace(`setBreakPointsRequest - Failed to open file: ${e}`);
        const fileUri = this.fileAccessor.resolvePathToUri(
          args.source.path ?? "",
        );
        log.trace("setBreakPointsRequest, target file: " + fileUri.toString());
        return undefined;
      });

    // If we couldn't find the file, or it wasn't a Q# file, just return
    if (!file || !isQsharpDocument(file)) {
      log.trace(`setBreakPointsResponse: %O`, response);
      this.sendResponse(response);
      return;
    }

    log.trace(`setBreakPointsRequest: looking`);
    this.breakpoints.set(file.uri.toString(), []);
    log.trace(
      `setBreakPointsRequest: files in cache %O`,
      this.breakpointLocations.keys(),
    );
    const locations = this.breakpointLocations.get(file.uri.toString()) ?? [];
    log.trace(`setBreakPointsRequest: got locations %O`, locations);
    // convert the request line/column to file offset for debugger
    const desiredBpOffsets: [
      lo: number,
      hi: number,
      isLineBreakpoint: boolean,
      uiLine: number,
    ][] = (args.breakpoints ?? [])
      .filter(
        (sourceBreakpoint) =>
          this.convertClientLineToDebugger(sourceBreakpoint.line) <
          file.lineCount,
      )
      .map((sourceBreakpoint) => {
        const isLineBreakpoint = !sourceBreakpoint.column;
        const line = this.convertClientLineToDebugger(sourceBreakpoint.line);
        const lineRange = file.lineAt(line).range;
        const startCol = sourceBreakpoint.column
          ? this.convertClientColumnToDebugger(sourceBreakpoint.column)
          : lineRange.start.character;
        const startPos = new vscode.Position(line, startCol);
        const startOffset = file.offsetAt(startPos);
        const endOffset = file.offsetAt(lineRange.end);

        return [
          startOffset,
          endOffset,
          isLineBreakpoint,
          sourceBreakpoint.line,
        ];
      });

    // We should probably ensure we don't return duplicate
    // spans from the debugger, but for now we'll just filter them out
    const uniqOffsets: [
      lo: number,
      hi: number,
      isLineBreakpoint: boolean,
      uiLine: number,
    ][] = [];
    for (const bpOffset of desiredBpOffsets) {
      if (
        uniqOffsets.findIndex(
          (u) => u[0] == bpOffset[0] && u[1] == bpOffset[1],
        ) == -1
      ) {
        uniqOffsets.push(bpOffset);
      }
    }
    // Now that we have the mapped breakpoint span, get the actual breakpoints
    // with corresponding ids from the debugger
    const bps = [];

    for (const bpOffset of uniqOffsets) {
      const lo = bpOffset[0];
      const isLineBreakpoint = bpOffset[2];
      const uiLine = bpOffset[3];
      // we can quickly filter out any breakpoints that are outside of the
      // desired line
      const matchingLocations = locations.filter((location) => {
        return location.uiLocation.line == uiLine;
      });
      // Now if the breakpoint is a line breakpoint, we can just use the first
      // matching location. Otherwise, we need to check if the desired column
      // is within the range of the location.
      for (const location of matchingLocations) {
        if (isLineBreakpoint) {
          //
          bps.push(location.breakpoint);
          break;
        } else {
          // column bp just has end of selection or cursor location in lo
          if (
            location.fileLocation.startOffset <= lo &&
            lo <= location.fileLocation.endOffset
          ) {
            bps.push(location.breakpoint);
            break;
          }
        }
      }
    }

    // Update our breakpoint list for the given file
    this.breakpoints.set(file.uri.toString(), bps);

    response.body = {
      breakpoints: bps,
    };

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
    args: DebugProtocol.StackTraceArguments,
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
                "stackTraceRequest, target file: " + fileUri.toString(),
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
                "qsharp-adapter-data",
              ),
              this.convertDebuggerLineToClient(start_pos.line),
              this.convertDebuggerColumnToClient(start_pos.character),
            );
            sf.endLine = this.convertDebuggerLineToClient(end_pos.line);
            sf.endColumn = this.convertDebuggerColumnToClient(
              end_pos.character,
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
                "qsharp-adapter-data",
              ) as DebugProtocol.Source;
              const sf = new StackFrame(
                id,
                f.name,
                source as Source,
                this.convertDebuggerLineToClient(start_pos.line),
                this.convertDebuggerColumnToClient(start_pos.character),
              );
              sf.endLine = this.convertDebuggerLineToClient(end_pos.line);
              sf.endColumn = this.convertDebuggerColumnToClient(
                end_pos.character,
              );

              return sf as DebugProtocol.StackFrame;
            } catch (e: any) {
              log.warn(e.message);
              return new StackFrame(
                id,
                f.name,
                undefined,
                undefined,
                undefined,
              );
            }
          }
        })
        .filter(filterUndefined),
    );
    const stackFrames = mappedStackFrames.reverse();
    stackFrames.push(
      new StackFrame(0, "entry", undefined) as DebugProtocol.StackFrame,
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
    request?: DebugProtocol.Request,
  ): void {
    log.trace(`disconnectRequest: %O`, args);
    this.debugService.terminate();
    this.sendResponse(response);
    log.trace(`disconnectResponse: %O`, response);
  }

  protected scopesRequest(
    response: DebugProtocol.ScopesResponse,
    args: DebugProtocol.ScopesArguments,
  ): void {
    log.trace(`scopesRequest: %O`, args);
    response.body = {
      scopes: [
        new Scope(
          "Quantum State",
          this.variableHandles.create("quantum"),
          true,
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
    request?: DebugProtocol.Request,
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
    location: DebugProtocol.BreakpointLocation,
  ): DebugProtocol.Breakpoint {
    const verified = true;
    const bp = new Breakpoint(verified) as DebugProtocol.Breakpoint;
    bp.id = id;
    bp.line = location.line;
    bp.column = location.column;
    bp.endLine = location.endLine;
    bp.endColumn = location.endColumn;
    return bp;
  }

  private writeToStdOut(message: string): void {
    const evt: DebugProtocol.OutputEvent = new OutputEvent(
      `${message}\n`,
      "stdout",
    );
    this.sendEvent(evt);
  }

  private writeToDebugConsole(message: string): void {
    const evt: DebugProtocol.OutputEvent = new OutputEvent(
      `${message}\n`,
      "console",
    );
    this.sendEvent(evt);
  }
}
