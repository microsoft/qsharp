// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import * as azqTools from "../copilot/azqTools";
import { ToolState } from "../copilot/tools";
import { QSharpTools } from "./qsharpTools";

// state
const workspaceState: ToolState = {};
let qsharpTools: QSharpTools | undefined;

const toolDefinitions: {
  name: string;
  tool: (input: any) => Promise<any>;
  confirm?: (input: any) => vscode.PreparedToolInvocation;
}[] = [
  // match these to the "languageModelTools" entries in package.json
  {
    name: "azure-quantum-get-jobs",
    tool: async (input) =>
      (await azqTools.getJobs(workspaceState, input)).result,
  },
  {
    name: "azure-quantum-get-job",
    tool: async (input: { job_id: string }) =>
      (await azqTools.getJob(workspaceState, input)).result,
  },
  {
    name: "azure-quantum-connect-to-workspace",
    tool: async () =>
      (await azqTools.connectToWorkspace(workspaceState)).result,
  },
  {
    name: "azure-quantum-download-job-results",
    tool: async (input: { job_id: string }) =>
      (await azqTools.downloadJobResults(workspaceState, input)).result,
  },
  {
    name: "azure-quantum-get-workspaces",
    tool: async () => (await azqTools.getWorkspaces()).result,
  },
  {
    name: "azure-quantum-submit-to-target",
    tool: async (input: {
      job_name: string;
      target_id: string;
      number_of_shots: number;
    }) => (await azqTools.submitToTarget(workspaceState, input, false)).result,
    confirm: (input: {
      job_name: string;
      target_id: string;
      number_of_shots: number;
    }): vscode.PreparedToolInvocation => ({
      confirmationMessages: {
        title: "Submit Azure Quantum job",
        message: `Submit job "${input.job_name}" to ${input.target_id} for ${input.number_of_shots} shots?`,
      },
    }),
  },
  {
    name: "azure-quantum-get-active-workspace",
    tool: async () =>
      (await azqTools.getActiveWorkspace(workspaceState)).result,
  },
  {
    name: "azure-quantum-set-active-workspace",
    tool: async (input: { workspace_id: string }) =>
      (await azqTools.setActiveWorkspace(workspaceState, input)).result,
  },
  {
    name: "azure-quantum-get-providers",
    tool: async () => (await azqTools.getProviders(workspaceState)).result,
  },
  {
    name: "azure-quantum-get-target",
    tool: async (input: { target_id: string }) =>
      (await azqTools.getTarget(workspaceState, input)).result,
  },
  {
    name: "qsharp-run-program",
    tool: async (input: { filePath: string; shots: number }) =>
      await qsharpTools!.runProgram(input),
  },
];

export function registerLanguageModelTools(context: vscode.ExtensionContext) {
  qsharpTools = new QSharpTools(context.extensionUri);
  for (const { name, tool: fn, confirm: confirmFn } of toolDefinitions) {
    context.subscriptions.push(
      vscode.lm.registerTool(name, tool(fn, confirmFn)),
    );
  }
}

function tool(
  toolFn: (input: any) => Promise<any>,
  confirmFn?: (input: any) => vscode.PreparedToolInvocation,
): vscode.LanguageModelTool<any> {
  return {
    invoke: (options: vscode.LanguageModelToolInvocationOptions<any>) =>
      invokeTool(options.input, toolFn),
    prepareInvocation:
      confirmFn &&
      ((options: vscode.LanguageModelToolInvocationPrepareOptions<any>) =>
        confirmFn(options.input)),
  };
}

async function invokeTool(
  input: any,
  toolFn: (input: any) => Promise<any>,
): Promise<vscode.LanguageModelToolResult> {
  log.info("Invoking tool");

  const result = await toolFn(input);

  log.info("tool result", result);

  return {
    content: [new vscode.LanguageModelTextPart(JSON.stringify(result))],
  };
}
