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
} from "@vscode/debugadapter";

import { FileAccessor } from "../common";
import { DebugProtocol } from "@vscode/debugprotocol";
import { ICompiler, log } from "qsharp";
import { createDebugConsoleEventTarget } from "./output";
import { ILaunchRequestArguments } from "./types";

const ErrorProgramHasErrors = "program contains compile errors(s): cannot run.";
const SimulationCompleted = "Q# simulation completed.";
const ConfigurationDelayMS = 1000;
const LaunchDelayMS = 100;

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class QscDebugSession extends LoggingDebugSession {
  private _fileAccessor: FileAccessor;
  private _compiler: ICompiler;

  public constructor(fileAccessor: FileAccessor, compiler: ICompiler) {
    super();
    this._fileAccessor = fileAccessor;
    this._compiler = compiler;

    this.setDebuggerLinesStartAt1(false);
    this.setDebuggerColumnsStartAt1(false);
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

    /* Settings that we need to eventually support: */

    // make VS Code send cancel request
    response.body.supportsCancelRequest = false;

    // make VS Code use 'evaluate' when hovering over source
    response.body.supportsEvaluateForHovers = false;

    // make VS Code send the breakpointLocations request
    response.body.supportsBreakpointLocationsRequest = false;
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
  ) {
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

    const programText = await this._fileAccessor.readFileAsString(args.program);

    const diagnostics = await this._compiler.checkCode(programText);
    if (diagnostics.length > 0) {
      log.trace("compilation failed. sending error response");
      this.sendErrorResponse(response, {
        id: -1,
        format: ErrorProgramHasErrors,
        showUser: true,
      });
      this.sendResponse(response);
      return;
    }

    if (args.noDebug) {
      log.trace(`Running without debugging`);
      this.sendResponse(response);
      await delay(LaunchDelayMS);
      this.runWithoutDebugging(args);
    } else {
      log.trace(`Running with debugging`);
      this.sendResponse(response);
      // This is where we would start the debugger
      await delay(LaunchDelayMS);
      this.runWithoutDebugging(args);
    }
  }

  private async runWithoutDebugging(
    args: ILaunchRequestArguments
  ): Promise<void> {
    const programText = await this._fileAccessor.readFileAsString(args.program);

    const eventTarget = createDebugConsoleEventTarget();
    this._compiler
      .run(programText, args.entry ?? "", args.shots, eventTarget)
      .then(() => {
        vscode.debug.activeDebugConsole.appendLine("");
        vscode.debug.activeDebugConsole.appendLine(SimulationCompleted);
        this.sendEvent(new TerminatedEvent());
        this.sendEvent(new ExitedEvent(0));
      });
  }

  protected disconnectRequest(
    response: DebugProtocol.DisconnectResponse,
    args: DebugProtocol.DisconnectArguments,
    _request?: DebugProtocol.Request
  ): void {
    log.trace(
      `disconnectRequest suspend: ${args.suspendDebuggee}, terminate: ${args.terminateDebuggee}`
    );
  }
}
