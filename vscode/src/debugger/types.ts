// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

import { DebugProtocol } from "@vscode/debugprotocol";

/**
 * This interface describes the Q# specific launch attributes
 * (which are not part of the Debug Adapter Protocol).
 * The schema for these attributes lives in package.json.
 * The interface should always match this schema.
 */
export interface ILaunchRequestArguments
  extends DebugProtocol.LaunchRequestArguments {
  /** An absolute path to the "program" to debug. */
  program: vscode.Uri;
  /** Automatically stop target after launch at the entry expression */
  stopOnEntry?: boolean;
  /** Entry expression to execute overriding default entry behavior */
  entry?: string;
  /** Number of shots to execute on run */
  shots: number;
  /** enable logging the Debug Adapter Protocol */
  trace?: boolean;
  /** run without debugging */
  noDebug?: boolean;
}
