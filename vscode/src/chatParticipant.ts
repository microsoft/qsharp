// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getRandomGuid } from "./utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "./azure/auth";
import { fetchEventSource } from "./fetch";

const chatUrl =
  "https://westus3.aqa.canary.quantum.azure.com/api/chat/streaming";
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

type quantumChatRequest = {
  conversationId: string; // GUID
  messages: Array<{
    role: string; // e.g. "user"
    content: string;
  }>; // The actual question
  additionalContext: any;
  identifier: string;
};

type QuantumChatResponse = {
  ConversationId: string; // GUID,
  Role: string; // e.g. "assistant"
  Content?: string; // The full response
  Delta?: string; // The next response token
  FinishReason?: string; // e.g. "stop"|"content_filter"|"length"|null,
  EmbeddedData: any;
  Created: string; // e.g. "2021-09-29T17:00:00.000Z"
};

async function chatRequest(
  token: string,
  question: string,
  stream: vscode.ChatResponseStream,
  context?: string,
): Promise<vscode.ChatResult> {
  log.debug("Requesting response");
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
    identifier: "Quantum",
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

  try {
    log.debug("About to call ChatAPI with payload: ", payload);
    await fetchEventSource(chatUrl, {
      ...options,
      onMessage(ev) {
        const messageReceived: QuantumChatResponse = JSON.parse(ev.data);
        log.debug("Received message: ", messageReceived);
        if (messageReceived.Delta) stream.markdown(messageReceived.Delta);
      },
    });

    return Promise.resolve({});
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

  //let response: QuantumChatResponse;
  if (request.command == "samples") {
    if (request.prompt) {
      await chatRequest(
        msaChatSession.accessToken,
        "Please show me the Q# code for " + request.prompt,
        stream,
      );
    } else {
      await chatRequest(
        msaChatSession.accessToken,
        "Can you list the names of the quantum samples you could write if asked?",
        stream,
        "The main samples I know how to write are Bell state, Grovers, QRNG, hidden shift, Bernstein-Vazarani, Deutsch-Jozsa, superdense coding, and teleportation",
      );
    }
  } else if (request.command == "quantumNotebook") {
    stream.progress("Opening a new Quantum Notebook...");
    await vscode.commands.executeCommand("qsharp-vscode.createNotebook");
    return Promise.resolve({});
  } else {
    await chatRequest(msaChatSession.accessToken, request.prompt, stream);
  }
  if (token.isCancellationRequested) return Promise.reject("Request cancelled");

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
