import * as vscode from "vscode";
import TelemetryReporter from "@vscode/extension-telemetry";

// the application insights key (also known as instrumentation key)
const key = "no";

// telemetry reporter
let reporter;

export function initTelemetry(context: vscode.ExtensionContext) {
  console.log("initializing telemetry")
  // create telemetry reporter on extension activation
  reporter = new TelemetryReporter(key);
  // ensure it gets properly disposed. Upon disposal the events will be flushed
  context.subscriptions.push(reporter);
  reporter.sendTelemetryEvent(
    "sampleEvent",
    { stringProp: "some string" },
    { numericMeasure: 123 },
  );
}
