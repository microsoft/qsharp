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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type TelemetryEvent = { id: string; data?: any };
export type TelemetryCollector = (event: TelemetryEvent) => void;

let telemetryCollector: TelemetryCollector | null = null;

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
    this.onLevelChanged?.(globalThis.qscLogLevel);
  },
  onLevelChanged: null as ((level: number) => void) | null,
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
  /**
   * @param level - A number indicating severity: 1 = Error, 2 = Warn, 3 = Info, 4 = Debug, 5 = Trace
   * @param target - The area or component sending the messsage, e.g. "parser" (useful for filtering)
   * @param args - The format string and args to log, e.g. ["Index of %s is %i", str, index]
   */
  logWithLevel(level: number, target: string, ...args: any) {
    // Convert to a format string containing the target (if present)
    const [, ...trailingArgs] = args; // All but first element of args
    const outArgs = [`[%s] ${args[0]}`, target || "", ...trailingArgs];
    switch (level) {
      case 1:
        log.error(...outArgs);
        break;
      case 2:
        log.warn(...outArgs);
        break;
      case 3:
        log.info(...outArgs);
        break;
      case 4:
        log.debug(...outArgs);
        break;
      case 5:
        log.trace(...outArgs);
        break;
      default:
        log.error("Invalid logLevel: ", level);
    }
  },
  setTelemetryCollector(handler: TelemetryCollector) {
    telemetryCollector = handler;
  },
  logTelemetry(event: { id: string; data?: any }) {
    telemetryCollector?.(event);
  },
  isTelemetryEnabled() {
    return !!telemetryCollector;
  },
};
/* eslint-enable @typescript-eslint/no-explicit-any */

// Enable globally for easy interaction and debugging in live environments
globalThis.qscLog = log;
globalThis.qscLogLevel = globalThis.qscLogLevel || 0;
