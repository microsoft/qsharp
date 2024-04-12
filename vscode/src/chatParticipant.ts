// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getRandomGuid } from "./utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "./azure/auth";

const chatUrl = "https://canary.api.quantum.microsoft.com/api/chat/completions";
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

type quantumChatRequest = {
  conversationId: string; // GUID
  messages: Array<{
    role: string; // e.g. "user"
    content: string;
  }>; // The actual question
  additionalContext: any; // ?
};

type QuantumChatResponse = {
  role: string; // e.g. "assistant"
  content: string; // The actual answer
  embeddedData: any; // ?
};

async function chatRequest(
  token: string,
  question: string,
  context?: string,
): Promise<QuantumChatResponse> {
  const payload: quantumChatRequest = {
    conversationId: getRandomGuid(),
    messages: [
      {
        role: "user",
        content: question,
      },
    ],
    additionalContext: {
      qcomEnvironment: "Desktop",
    },
  };

  if (context) {
    payload.messages.unshift({
      role: "assistant",
      content: context,
    });
  }

  const options = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  };
  log.debug("About to call ChatAPI with payload: ", payload);

  try {
    const response = await fetch(chatUrl, options);
    log.debug("ChatAPI response status: ", response.statusText);

    const json = await response.json();
    log.debug("ChatAPI response payload: ", json);
    return json;
  } catch (error) {
    log.error("ChatAPI fetch failed with error: ", error);
    throw error;
  }
}

const requestHandler: vscode.ChatRequestHandler = async (
  request: vscode.ChatRequest,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  _context: vscode.ChatContext,
  stream: vscode.ChatResponseStream,
  token: vscode.CancellationToken,
): Promise<vscode.ChatResult> => {
  const msaChatSession = await getAuthSession(
    [scopes.chatApi, `VSCODE_TENANT:common`, `VSCODE_CLIENT_ID:${chatApp}`],
    getRandomGuid(),
  );
  if (!msaChatSession) throw Error("Failed to get MSA chat token");

  let response: QuantumChatResponse;
  if (request.command == "samples") {
    if (request.prompt) {
      response = await chatRequest(
        msaChatSession.accessToken,
        "Please show me the Q# code for " + request.prompt,
      );
    } else {
      response = await chatRequest(
        msaChatSession.accessToken,
        "Can you list the names of the quantum samples you could write if asked?",
        "The main samples I know how to write are Bell state, Grovers, QRNG, hidden shift, Bernstein-Vazarani, Deutsch-Jozsa, superdense coding, and teleportation",
      );
    }
  } else if (request.command == "quantumNotebook") {
    stream.progress("Opening a new Quantum Notebook...");
    await vscode.commands.executeCommand("qsharp-vscode.createNotebook");
    return Promise.resolve({});
  } else {
    response = await chatRequest(msaChatSession.accessToken, request.prompt);
  }
  if (token.isCancellationRequested) return Promise.reject("Request cancelled");

  stream.markdown(response.content);

  return Promise.resolve({});
};

export function activateChatParticipant(context: vscode.ExtensionContext) {
  const copilot = vscode.chat.createChatParticipant(
    "quantum.copilot",
    requestHandler,
  );

  copilot.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "resources",
    "copilotIcon.png",
  );

  // Register a chat agent
  context.subscriptions.push(copilot);
}
