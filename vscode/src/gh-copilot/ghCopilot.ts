import * as vscode from "vscode";
import * as azqTools from "../copilot/azqTools";
import { ToolState } from "../copilot/tools";
import { log } from "qsharp-lang";

const workspaceState: ToolState = {};

async function getJobs(
  options: vscode.LanguageModelToolInvocationOptions<{ lastNDays: number }>,
): Promise<any> {
  return (await azqTools.getJobs(workspaceState, options.input)).result;
}

async function proofreadCode(
  options: vscode.LanguageModelToolInvocationOptions<{ code: string }>,
) {
  return `\`\`\`qsharp
${options.input.code.replace(/q/g, "ℚ")}
\`\`\``;
}

async function findSample(
  options: vscode.LanguageModelToolInvocationOptions<{ description: string }>,
) {
  return `\`\`\`qsharp
use q = Qubit(); // ${options.input.description}
\`\`\``;
}

function tool(
  tool: (
    options: vscode.LanguageModelToolInvocationOptions<any>,
  ) => Promise<any>,
) {
  return {
    invoke: (options: vscode.LanguageModelToolInvocationOptions<any>) =>
      invokeTool(options, tool),
  };
}

async function invokeTool(
  options: vscode.LanguageModelToolInvocationOptions<any>,
  tool: (
    options: vscode.LanguageModelToolInvocationOptions<any>,
  ) => Promise<any>,
): Promise<vscode.LanguageModelToolResult> {
  log.info("Invoking tool");

  const result = await tool(options);

  log.info("tool result", result);

  return {
    content: [new vscode.LanguageModelTextPart(JSON.stringify(result))],
  };
}

async function qsharpCoding(
  options: vscode.LanguageModelToolInvocationOptions<undefined>,
) {
  return "The `namespace` keyword is deprecated in recent versions of Q#. Don't wrap code in a namespace.";
}

export function registerLanguageModelTools(context: vscode.ExtensionContext) {
  const tools: { name: string; tool: vscode.LanguageModelTool<any> }[] = [
    {
      name: "azure-quantum-get-jobs",
      tool: tool(getJobs),
    },
    { name: "qsharp-proofread-code", tool: tool(proofreadCode) },
    { name: "qsharp-find-sample", tool: tool(findSample) },
    { name: "qsharp-coding", tool: tool(qsharpCoding) },
  ];

  for (const { name, tool } of tools) {
    context.subscriptions.push(vscode.lm.registerTool(name, tool));
  }
}
