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
  BreakpointEvent,
  Thread,
  StackFrame,
  Source,
} from "@vscode/debugadapter";

import { FileAccessor } from "../common";
import { DebugProtocol } from "@vscode/debugprotocol";
import {
  BreakpointSpan,
  ICompiler,
  IDebugServiceWorker,
  log,
  Span,
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
  private _fileAccessor: FileAccessor;
  private _compiler: ICompiler;
  private _debugger: IDebugServiceWorker;
  private _config: vscode.DebugConfiguration;
  private _breakpointLocations: Map<string, BreakpointSpan[]>;
  private _breakpoints: Map<string, DebugProtocol.Breakpoint[]>;
  private _failed: boolean;

  public constructor(
    fileAccessor: FileAccessor,
    compiler: ICompiler,
    debugService: IDebugServiceWorker,
    config: vscode.DebugConfiguration
  ) {
    super();
    this._fileAccessor = fileAccessor;
    this._compiler = compiler;
    this._debugger = debugService;
    this._config = config;
    this._failed = false;
    this._breakpointLocations = new Map<string, BreakpointSpan[]>();
    this._breakpoints = new Map<string, DebugProtocol.Breakpoint[]>();
    this.setDebuggerLinesStartAt1(false);
    this.setDebuggerColumnsStartAt1(false);
  }

  public async init(): Promise<void> {
    const programText = await this._fileAccessor.readFileAsString(
      this._config.program
    );

    const loaded = await this._debugger.loadSource(
      this._config.program.path,
      programText
    );
    if (loaded) {
      const locations = await this._debugger.getBreakpoints(
        this._config.program.path
      );
      log.trace(
        `init breakpointLocations: ${JSON.stringify(locations, null, 2)}`
      );
      this._breakpointLocations.set(this._config.program.path, locations);
    } else {
      log.warn(`compilation failed.`);
      this._failed = true;
    }
  }

  /**
   * The 'initialize' request is the first request called by the frontend
   * to interrogate the features the debug adapter provides.
   */
  protected initializeRequest(
    response: DebugProtocol.InitializeResponse,
    _args: DebugProtocol.InitializeRequestArguments
  ): void {
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
    if (this._failed) {
      log.trace("compilation failed. sending error response");
      this.sendErrorResponse(response, {
        id: -1,
        format: ErrorProgramHasErrors,
        showUser: true,
      });
      this.sendResponse(response);
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

    if (args.noDebug) {
      log.trace(`Running without debugging`);
      await this.runWithoutDebugging(args);
    } else {
      log.trace(`Running with debugging`);
      await this.runWithDebugging(args);
    }
    log.trace(`sending launchRequest response`);
    this.sendResponse(response);
  }
  private async runWithDebugging(
    _args: ILaunchRequestArguments
  ): Promise<void> {
    const bps = this.get_breakpoint_ids();
    this.run(bps);
  }
  private async run(bps: Uint32Array): Promise<void> {
    const eventTarget = createDebugConsoleEventTarget();
    const res = await this._debugger
      .evalContinue(bps, eventTarget)
      .catch((e) => {
        log.info(`ending session due to error: ${e}`);
        vscode.debug.activeDebugConsole.appendLine("");
        vscode.debug.activeDebugConsole.appendLine(SimulationCompleted);
        this.sendEvent(new TerminatedEvent());
        this.sendEvent(new ExitedEvent(0));
      });

    if (res) {
      log.trace(`raising breakpoint event`);
      const evt = new StoppedEvent(
        "breakpoint",
        QscDebugSession.threadID
      ) as DebugProtocol.StoppedEvent;
      evt.body.hitBreakpointIds = [res];
      this.sendEvent(evt);

      this.sendEvent(
        new BreakpointEvent("changed", {
          verified: true,
          id: res,
        } as DebugProtocol.Breakpoint)
      );
    } else {
      this.endSession(`ending session`);
    }
  }

  private endSession(message: string) {
    log.trace(message);
    vscode.debug.activeDebugConsole.appendLine("");
    vscode.debug.activeDebugConsole.appendLine(SimulationCompleted);
    this.sendEvent(new TerminatedEvent());
    this.sendEvent(new ExitedEvent(0));
  }

  private async runWithoutDebugging(
    args: ILaunchRequestArguments
  ): Promise<void> {
    const bps = new Uint32Array();
    for (let i = 0; i < args.shots; i++) {
      this.run(bps);
    }
  }

  private get_breakpoint_ids(): Uint32Array {
    const bps: number[] = [];
    for (const bp of this._breakpoints.get(this._config.program.path) ?? []) {
      if (bp && bp.id) {
        bps.push(bp.id);
      }
    }
    const v = new Uint32Array(bps);
    return v;
  }
  protected async continueRequest(
    response: DebugProtocol.ContinueResponse,
    args: DebugProtocol.ContinueArguments
  ): Promise<void> {
    log.trace(`continueRequest: ${JSON.stringify(args, null, 2)}`);
    const bps = this.get_breakpoint_ids();

    log.trace(`sending continue response`);
    this.sendResponse(response);

    const eventTarget = createDebugConsoleEventTarget();
    await this._debugger.evalContinue(bps, eventTarget).then(
      (res) => {
        if (res) {
          log.trace(`raising breakpoint event`);
          this.sendEvent(
            new StoppedEvent("breakpoint", QscDebugSession.threadID)
          );
          this.sendEvent(
            new BreakpointEvent("changed", {
              verified: true,
              id: res,
            } as DebugProtocol.Breakpoint)
          );
        } else {
          this.endSession(`ending session`);
        }
      },
      (e) => {
        log.info(`Runtime error: ${e}`);
        this.endSession(`ending session`);
      }
    );
  }

  protected async breakpointLocationsRequest(
    response: DebugProtocol.BreakpointLocationsResponse,
    args: DebugProtocol.BreakpointLocationsArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`breakpointLocationsRequest: ${JSON.stringify(args, null, 2)}`);

    response.body = {
      breakpoints: [],
    };

    const fileUri = vscode.Uri.parse(args.source.path ?? "", false);

    const file = vscode.workspace.textDocuments.find(
      (td) => td.uri.path === fileUri.path
    );
    if (!file) {
      for (const td of vscode.workspace.textDocuments) {
        log.trace("breakpointLocationsRequest: potential file" + td.uri.path);
      }
      log.trace("breakpointLocationsRequest: target file" + fileUri.path);
    }
    if (fileUri && file) {
      // Map request start/end line/column to file offset for debugger
      const lineRange = file.lineAt(args.line).range;
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
      /*const bps =
        this._breakpointLocations
          .get(fileUri.path)
          ?.filter((bp) => startOffset <= bp.lo && bp.hi <= endOffset) ?? [];*/
      const bps = this._breakpointLocations.get(fileUri.path) ?? [];
      log.trace(
        `breakpointLocationsRequest: candidates ${JSON.stringify(bps, null, 2)}`
      );

      // must map the debugger breakpoints back to the client breakpoint locations
      const bls = bps.map((bps) => {
        const startPos = file.positionAt(bps.lo);
        const endPos = file.positionAt(bps.hi);
        const bp: DebugProtocol.BreakpointLocation = {
          line: this.convertDebuggerLineToClient(startPos.line),
          column: this.convertDebuggerColumnToClient(startPos.character),
          endLine: this.convertDebuggerLineToClient(endPos.line),
          endColumn: this.convertDebuggerColumnToClient(endPos.character),
        };
        return bp;
      });
      log.trace(
        `breakpointLocationsRequest: mapped ${JSON.stringify(bls, null, 2)}`
      );
      response.body = {
        breakpoints: bls,
      };
    }
    log.trace(
      `breakpointLocationsResponse: ${JSON.stringify(response, null, 2)}`
    );
    this.sendResponse(response);
  }

  protected async setBreakPointsRequest(
    response: DebugProtocol.SetBreakpointsResponse,
    args: DebugProtocol.SetBreakpointsArguments,
    request?: DebugProtocol.Request
  ): Promise<void> {
    log.trace(`setBreakPointsRequest: ${JSON.stringify(args, null, 2)}`);

    const fileUri = vscode.Uri.parse(args.source.path ?? "", false);

    const file = vscode.workspace.textDocuments.find(
      (td) => td.uri.path === fileUri.path
    );
    if (!file) {
      for (const td of vscode.workspace.textDocuments) {
        log.trace("setBreakPointsRequest: potential file" + td.uri.path);
      }
      log.trace("setBreakPointsRequest: target file" + fileUri.path);
    }

    if (fileUri && file) {
      log.trace(`setBreakPointsRequest: looking`);
      this._breakpoints.set(fileUri.path, []);
      log.trace(
        `setBreakPointsRequest: files in cache ${JSON.stringify(
          this._breakpointLocations.keys(),
          null,
          2
        )}`
      );
      const locations = this._breakpointLocations.get(fileUri.path) ?? [];
      log.trace(
        `setBreakPointsRequest: got locations ${JSON.stringify(
          locations,
          null,
          2
        )}`
      );
      // convert the request line/column to file offset for debugger
      const bpOffsets = (args.breakpoints ?? []).map((sourceBreakpoint) => {
        const line = this.convertClientLineToDebugger(sourceBreakpoint.line);
        const lineRange = file.lineAt(line).range;
        const startCol = sourceBreakpoint.column
          ? this.convertClientColumnToDebugger(sourceBreakpoint.column)
          : lineRange.start.character;
        const startPos = new vscode.Position(line, startCol);
        const startOffset = file.offsetAt(startPos);
        const endOffset = file.offsetAt(lineRange.end);

        return new Span(startOffset, endOffset);
      });

      // We should probably ensure we don't return duplicate
      // spans from the debugger, but for now we'll just filter them out
      const uniqOffsets = [];
      for (const bpOffset of bpOffsets) {
        if (
          uniqOffsets.findIndex(
            (u) => u.lo == bpOffset.lo && u.hi == bpOffset.hi
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
          if (bpOffset.lo <= location.lo && location.hi <= bpOffset.hi) {
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
      this._breakpoints.set(fileUri.path, bps);

      response.body = {
        breakpoints: bps,
      };
    }
    log.trace(`setBreakPointsResponse: ${JSON.stringify(response, null, 2)}`);
    this.sendResponse(response);
  }

  protected threadsRequest(response: DebugProtocol.ThreadsResponse): void {
    log.trace(`threadRequest`);
    response.body = {
      threads: [new Thread(QscDebugSession.threadID, "thread 1")],
    };
    log.trace(`threadResponse: ${JSON.stringify(response, null, 2)}`);
    this.sendResponse(response);
  }

  protected async stackTraceRequest(
    response: DebugProtocol.StackTraceResponse,
    args: DebugProtocol.StackTraceArguments
  ): Promise<void> {
    log.trace(`stackTraceRequest: ${JSON.stringify(args, null, 2)}`);
    const debuggerStackFrames = await this._debugger.getStackFrames();
    log.trace(`frames: ${JSON.stringify(debuggerStackFrames, null, 2)}`);
    const filterUndefined = <V>(value: V | undefined): value is V =>
      value != null;
    const mappedStackFrames: StackFrame[] = debuggerStackFrames
      .map((f, id) => {
        const fileUri = vscode.Uri.parse(f.path, false);
        log.trace(`frames: fileUri${JSON.stringify(fileUri, null, 2)}`);
        const file = vscode.workspace.textDocuments.find(
          (td) => td.uri.path === fileUri.path
        );
        if (!file) {
          // This file isn't part of the workspace, so we'll
          // create a dummy source for it. In the future, we
          // can use source id to load the file from the compiler
          // if it is part of the std lib.
          const source = new Source(
            f.name,
            undefined,
            0,
            undefined,
            "qsharp-adapter-data"
          ) as DebugProtocol.Source;
          source.presentationHint = "deemphasize";

          const sf = new StackFrame(id, f.name, source as Source);

          return sf as DebugProtocol.StackFrame;
        }
        log.trace(`frames: file ${JSON.stringify(file, null, 2)}`);
        const start_pos = file.positionAt(f.lo);
        const end_pos = file.positionAt(f.hi);
        const sf: DebugProtocol.StackFrame = new StackFrame(
          id,
          f.name,
          new Source(
            file.uri.toString(true),
            file.uri.toString(true),
            undefined,
            undefined,
            "qsharp-adapter-data"
          ),
          this.convertDebuggerLineToClient(start_pos.line),
          this.convertDebuggerColumnToClient(start_pos.character)
        );
        sf.endLine = this.convertDebuggerLineToClient(end_pos.line);
        sf.endColumn = this.convertDebuggerColumnToClient(end_pos.character);
        return sf;
      })
      .filter(filterUndefined);
    const stackFrames = mappedStackFrames.reverse();
    stackFrames.push(
      new StackFrame(0, "entry", undefined) as DebugProtocol.StackFrame
    );
    response.body = {
      stackFrames: stackFrames,
      totalFrames: stackFrames.length,
    };

    log.trace(`stackTraceResponse: ${JSON.stringify(response, null, 2)}`);
    this.sendResponse(response);
  }

  protected disconnectRequest(
    response: DebugProtocol.DisconnectResponse,
    args: DebugProtocol.DisconnectArguments,
    request?: DebugProtocol.Request
  ): void {
    log.trace(`disconnectRequest: ${JSON.stringify(args, null, 2)}`);
    this._debugger.terminate();
    this.sendResponse(response);
    log.trace(`disconnectResponse: ${JSON.stringify(response, null, 2)}`);
  }

  protected scopesRequest(
    response: DebugProtocol.ScopesResponse,
    args: DebugProtocol.ScopesArguments
  ): void {
    log.trace(`scopesRequest: ${JSON.stringify(args, null, 2)}`);
    response.body = {
      scopes: [],
    };
    log.trace(`scopesResponse: ${JSON.stringify(response, null, 2)}`);
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
}
