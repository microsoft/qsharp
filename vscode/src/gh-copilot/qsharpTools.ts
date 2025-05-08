// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log, QscEventTarget } from "qsharp-lang";
import vscode from "vscode";
import { loadCompilerWorker } from "../common";
import { getPauliNoiseModel } from "../config";
import { CopilotToolError } from "../copilot/tools";
import { getProgramForDocument } from "../programConfig";
import { sendMessageToPanel } from "../webviewPanel.js";
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
  async runProgram(input: {
    filePath: string;
    shots?: number;
  }): Promise<string> {
    const shots = input.shots ?? 1;
    try {
      const docUri = vscode.Uri.file(input.filePath);
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
      const measurementResults: Record<string, number> = {};

      // Capture standard outputs and results
      evtTarget.addEventListener("Message", (evt) => {
        outputResults.push(evt.detail);
      });

      evtTarget.addEventListener("Result", (evt) => {
        // Handle both string results and diagnostics
        if (!evt.detail.success && evt.detail.value.message !== undefined) {
          outputResults.push(JSON.stringify(evt.detail.value));
        } else {
          const result = `${evt.detail.value}`;
          outputResults.push(result);

          // Collect measurement results for histogram if multiple shots
          if (shots > 1) {
            if (measurementResults[result]) {
              measurementResults[result]++;
            } else {
              measurementResults[result] = 1;
            }
          }
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
          // If shots > 1, display histogram
          if (shots > 1) {
            const buckets: Array<[string, number]> =
              Object.entries(measurementResults);
            const histogram = {
              buckets,
              shotCount: shots,
            };

            const panelId = programResult.programConfig.projectName;

            // Show the histogram
            sendMessageToPanel(
              { panelType: "histogram", id: panelId },
              true, // reveal the panel
              histogram,
            );

            return `Program executed successfully with ${shots} shots.\n Results: ${JSON.stringify(histogram)}`;
          }

          return `Program executed successfully.\nOutput:\n${outputResults.join("\n")}`;
        }
      } catch {
        throw new CopilotToolError(
          `Program execution failed: ${outputResults.join("\n")}`,
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
  } /**
   * Generates a circuit diagram for the specified Q# file
   */
  async generateCircuit(input: { filePath: string }): Promise<string> {
    try {
      // Get the Q# document from the file path
      const docUri = vscode.Uri.file(input.filePath);
      if (!docUri) {
        throw new CopilotToolError(
          "Invalid file path. Please provide a valid path to a Q# file.",
        );
      }

      // Check if the program can be compiled
      const programResult = await getProgramForDocument(docUri);
      if (!programResult.success) {
        throw new CopilotToolError(
          `Cannot generate circuit: ${programResult.errorMsg}`,
        );
      }

      // TODO: pass file path
      // Generate the circuit diagram (without specifying an operation - will show all)
      await vscode.commands.executeCommand("qsharp-vscode.showCircuit");

      return "Circuit diagram generated and displayed in the circuit panel.";
    } catch (e) {
      log.error("Failed to generate circuit diagram. ", e);
      throw new CopilotToolError(
        "Failed to generate circuit diagram: " +
          (e instanceof Error ? e.message : String(e)),
      );
    }
  }

  /**
   * Runs the resource estimator on the specified Q# file
   */
  async runResourceEstimator(input: { filePath: string }): Promise<string> {
    try {
      // Get the Q# document from the file path
      const docUri = vscode.Uri.file(input.filePath);
      if (!docUri) {
        throw new CopilotToolError(
          "Invalid file path. Please provide a valid path to a Q# file.",
        );
      }

      // Check if the program can be compiled
      const programResult = await getProgramForDocument(docUri);
      if (!programResult.success) {
        throw new CopilotToolError(
          `Cannot run resource estimator: ${programResult.errorMsg}`,
        );
      }

      // TODO: pass file path
      // Call the showRe command from the VS Code extension
      await vscode.commands.executeCommand("qsharp-vscode.showRe");

      return "Resource estimation started. Results will be displayed in the resource estimator panel.";
    } catch (e) {
      log.error("Failed to run resource estimator. ", e);
      throw new CopilotToolError(
        "Failed to run resource estimator: " +
          (e instanceof Error ? e.message : String(e)),
      );
    }
  }
}
