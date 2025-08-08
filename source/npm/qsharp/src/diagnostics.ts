// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "./log.js";
import type { IQSharpError, IRange } from "../lib/web/qsc_wasm.js";

/**
 * Public error type for most `qsharp-lang` functions.
 *
 * This is typically thrown by functions that deal with compiling and
 * running code. Contains one or more VS Code-like diagnostics.
 */
export class QdkDiagnostics extends Error {
  constructor(public readonly diagnostics: IQSharpError[]) {
    const message = shortMessage(diagnostics);
    super(message);
    this.name = "QdkDiagnostics";
  }
}

/**
 * Wrapper around QDK WASM functions to convert exceptions to a more ergonomic type.
 *
 * Many of the WASM functions throw exceptions that are of type `string`,
 * which are really just JSON-serialized `IQSharpError[]`s.
 *
 * This function converts those exceptions into `QdkDiagnostics` instances
 * that properly inherit from `Error`.
 */
export async function callAndTransformExceptions<T>(fn: () => Promise<T>) {
  try {
    return await fn();
  } catch (e: unknown) {
    const QdkDiagnostics = tryParseQdkDiagnostics(e);
    if (QdkDiagnostics) {
      throw QdkDiagnostics;
    }
    throw e;
  }
}

/**
 * If the error is a string containing JSON-serialized `IQSharpError[]`,
 * creates a `QdkDiagnostics` instance from it.
 */
function tryParseQdkDiagnostics(e: unknown): QdkDiagnostics | undefined {
  if (typeof e === "string") {
    try {
      const errors = JSON.parse(e);
      // Check for the shape of IQSharpError[]
      if (
        Array.isArray(errors) &&
        errors.length > 0 &&
        errors[0].document &&
        errors[0].diagnostic
      ) {
        return new QdkDiagnostics(errors);
      }
    } catch {
      // Couldn't parse the error as JSON.
      log.warn(`could not parse error string ${e}`);
    }
  }

  return undefined;
}

/**
 * Constructs a human-readable message from the first error.
 */
function shortMessage(errors: IQSharpError[]) {
  const error = errors[0];

  return `${error.diagnostic.message}${friendlyLocation(
    error.document,
    error.diagnostic.range,
  )}`;
}

/**
 * Constructs a human-readable location string for an error.
 */
function friendlyLocation(uriOrName: string, range: IRange) {
  if (uriOrName === "<project>") {
    return "";
  }
  // Don't make any assumptions about the format of uriOrName,
  // it could be a file path, a URI with an arbitrary scheme, or just a name.
  // If it contains slashes, we assume it's a path and extract the basename.
  // Best effort, for display only.
  const lastSlash = Math.max(
    uriOrName.lastIndexOf("/"),
    uriOrName.lastIndexOf("\\"),
  );
  const basename =
    lastSlash >= 0 ? uriOrName.substring(lastSlash + 1) : uriOrName;

  // avoid printing :1:1 since that's the default if the original error didn't specify a range
  const lineColumn =
    range.start.line > 0 || range.start.character > 0
      ? `:${range.start.line + 1}:${range.start.character + 1}`
      : "";

  return ` at ${basename}${lineColumn}`;
}
