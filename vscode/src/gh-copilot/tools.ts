// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import * as azqTools from "../copilot/azqTools";
import { ToolState } from "../copilot/tools";
import { log } from "qsharp-lang";

const toolDefinitions: {
  name: string;
  tool: (input: any) => Promise<any>;
  confirm?: (input: any) => vscode.PreparedToolInvocation;
}[] = [
  // match these to the "languageModelTools" entries in package.json
  { name: "azure-quantum-get-jobs", tool: getJobs },
  { name: "azure-quantum-get-job", tool: getJob },
  { name: "azure-quantum-connect-to-workspace", tool: connectToWorkspace },
  { name: "azure-quantum-download-job-results", tool: downloadJobResults },
  { name: "azure-quantum-get-workspaces", tool: getWorkspaces },
  {
    name: "azure-quantum-submit-to-target",
    tool: submitToTarget,
    confirm: submitToTargetConfirm,
  },
  { name: "azure-quantum-get-active-workspace", tool: getActiveWorkspace },
  { name: "azure-quantum-set-active-workspace", tool: setActiveWorkspace },
  { name: "azure-quantum-get-providers", tool: getProviders },
  { name: "azure-quantum-get-target", tool: getTarget },
];

const workspaceState: ToolState = {};

async function getJobs(input: { lastNDays: number }): Promise<any> {
  return (await azqTools.getJobs(workspaceState, input)).result;
}

async function getJob(input: { job_id: string }): Promise<any> {
  return (await azqTools.getJob(workspaceState, input)).result;
}

async function connectToWorkspace(): Promise<any> {
  return (await azqTools.connectToWorkspace(workspaceState)).result;
}

async function downloadJobResults(input: { job_id: string }): Promise<any> {
  return (await azqTools.downloadJobResults(workspaceState, input)).result;
}

async function getWorkspaces(): Promise<any> {
  return (await azqTools.getWorkspaces()).result;
}

async function submitToTarget(input: {
  job_name: string;
  target_id: string;
  number_of_shots: number;
}): Promise<any> {
  return (await azqTools.submitToTarget(workspaceState, input, false)).result;
}

function submitToTargetConfirm(input: {
  job_name: string;
  target_id: string;
  number_of_shots: number;
}): vscode.PreparedToolInvocation {
  return {
    confirmationMessages: {
      title: "Submit Azure Quantum job",
      message: `Submit job "${input.job_name}" to ${input.target_id} for ${input.number_of_shots} shots?`,
    },
  };
}

async function getActiveWorkspace(): Promise<any> {
  return (await azqTools.getActiveWorkspace(workspaceState)).result;
}

async function setActiveWorkspace(input: {
  workspace_id: string;
}): Promise<any> {
  return (await azqTools.setActiveWorkspace(workspaceState, input)).result;
}

async function getProviders(): Promise<any> {
  return (await azqTools.getProviders(workspaceState)).result;
}

async function getTarget(input: { target_id: string }): Promise<any> {
  return (await azqTools.getTarget(workspaceState, input)).result;
}

export function registerLanguageModelTools(context: vscode.ExtensionContext) {
  for (const { name, tool: fn, confirm: confirmFn } of toolDefinitions) {
    context.subscriptions.push(
      vscode.lm.registerTool(name, tool(fn, confirmFn)),
    );
  }
}

///// MISC

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
