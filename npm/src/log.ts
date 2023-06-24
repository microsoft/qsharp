// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Logging infrastructure for JavaScript environments (e.g. browser and node.js)
//
// Ideally this should be the only module to have global side effects and run code
// on module load (i.e. other modules should consist almost entirely of declarations
// and exports at the top level), which means it is configurable and usable from
// the host environment after import resolution and before other logic runs.

declare global {
  // Align with VS Code names (but not level numbers)
  // 0 = off
  // 1 = error
  // 2 = warn
  // 3 = info
  // 4 = debug (called 'verbose' in VS Code)
  // 5 = trace
  // Note this also aligns with the Rust log crate macros/levels
  // See https://docs.rs/log/latest/log/
  var qscLogLevel: number; // eslint-disable-line no-var
  var qscLog: typeof log; // eslint-disable-line no-var
  var qscGitHash: string; // eslint-disable-line no-var
}

export type LogLevel = "off" | "error" | "warn" | "info" | "debug" | "trace";

export const log = {
  setLogLevel(level: LogLevel | number) {
    if (typeof level === "string") {
      // Convert to number
      const lowerLevel = level.toLowerCase();
      const levels = ["off", "error", "warn", "info", "debug", "trace"];
      let newLevel = 0;
      levels.forEach((name, idx) => {
        if (name === lowerLevel) newLevel = idx;
      });
      globalThis.qscLogLevel = newLevel;
    } else {
      globalThis.qscLogLevel = level;
    }
  },
  getLogLevel(): number {
    return globalThis.qscLogLevel || 0;
  },
  error(...args: any) {
    if (qscLogLevel >= 1) console.error(...args);
  },
  warn(...args: any) {
    if (qscLogLevel >= 2) console.warn(...args);
  },
  info(...args: any) {
    if (qscLogLevel >= 3) console.info(...args);
  },
  debug(...args: any) {
    if (qscLogLevel >= 4) console.debug(...args);
  },
  trace(...args: any) {
    // console.trace in JavaScript just writes a stack trace at info level, so use 'debug'
    if (qscLogLevel >= 5) console.debug(...args);
  },
  never(val: never) {
    // Utility function to ensure exhaustive type checking. See https://stackoverflow.com/a/39419171
    log.error("Exhaustive type checking didn't account for: %o", val);
  },
};

// Default to the 'error' level for logging
if (typeof globalThis.qscLogLevel === "undefined") {
  log.setLogLevel("error");
}

// Enable globally for easy interaction and debugging in live environments
globalThis.qscLog = log;
