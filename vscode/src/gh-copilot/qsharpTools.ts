// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log, QscEventTarget, VSDiagnostic } from "qsharp-lang";
import vscode from "vscode";
import { loadCompilerWorker } from "../common";
import { getPauliNoiseModel } from "../config";
import { CopilotToolError } from "../copilot/tools";
import {
  getActiveQSharpDocumentUri,
  getProgramForDocument,
} from "../programConfig";
// import  { ConfigurationTarget } from "vscode";

export class QSharpTools {
  constructor(private extensionUri: vscode.Uri) {
    // some test code to try out configuration updates
    // log.info("QSharpTools initialized");
    // const cfg = vscode.workspace.getConfiguration("chat");
    // const val = cfg.inspect("agent")?.globalValue;
    // cfg.update("agent.enabled", true, ConfigurationTarget.Global).then(
    //   () => {
    //     const lastVal = cfg.inspect("agent")?.globalValue;
    //     log.info("Agent config value: ", lastVal);
    //   },
    //   (e) => {
    //     log.error("Failed to update agent config", e);
    //   },
    // );
    // log.info("Agent config value: ", val);
  }

  /**
   * Runs the current Q# program in the editor
   */
  async runProgram({ shots = 1 }: { shots?: number }): Promise<string> {
    try {
      // Get the active Q# document
      const docUri = getActiveQSharpDocumentUri();
      if (!docUri) {
        throw new CopilotToolError(
          "No active Q# document found. Please open a Q# file first.",
        );
      }

      // Check if the program can be compiled
      const programResult = await getProgramForDocument(docUri);
      if (!programResult.success) {
        throw new CopilotToolError(
          `Cannot run the program: ${programResult.errorMsg}`,
        );
      }

      // Create an event target to capture results
      const evtTarget = new QscEventTarget(true);
      const outputResults: string[] = [];

      // Capture standard outputs and results
      evtTarget.addEventListener("Message", (evt) => {
        outputResults.push(evt.detail);
      });

      evtTarget.addEventListener("Result", (evt) => {
        // Handle both string results and diagnostics
        if ((evt.detail.value as VSDiagnostic).message !== undefined) {
          outputResults.push((evt.detail.value as VSDiagnostic).message);
        } else {
          outputResults.push(`${evt.detail.value}`);
        }
      });

      const worker = await loadCompilerWorker(this.extensionUri!);

      try {
        // Get the noise model (if configured)
        const noise = getPauliNoiseModel();

        // Run the program with the compiler worker
        await worker.runWithPauliNoise(
          programResult.programConfig,
          "", // No specific entry expression
          shots,
          noise,
          evtTarget,
        );

        // Format and return the results
        if (outputResults.length === 0) {
          return "Program executed successfully but produced no output.";
        } else {
          return `Program executed successfully.\nOutput:\n${outputResults.join("\n")}`;
        }
      } catch (e) {
        throw new CopilotToolError(
          `Program execution failed: ${e instanceof Error ? e.message : String(e)}`,
        );
      } finally {
        // Always terminate the worker when done
        worker.terminate();
      }
    } catch (e) {
      log.error("Failed to run program. ", e);
      throw new CopilotToolError(
        "Failed to run the Q# program: " +
          (e instanceof Error ? e.message : String(e)),
      );
    }
  }
}
