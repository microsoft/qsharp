// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getRandomGuid } from "./utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "./azure/auth";
import { fetchEventSource } from "./fetch";

const chatUrl = "https://westus3.aqa.quantum.azure.com/api/chat/streaming";
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

const latexContext = String.raw`Please ensure any LaTeX code is enclosed in single or double dollar signs, e.g. $x^2$ or $$x^2$$ , and not escaped parentheses, e.g. \(x^2\).`;

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

// Consider using a generator function to handle the stream
export type CopilotStreamCallback = (mdFragment: string, done: boolean) => void;

export async function makeChatRequest(
  question: string,
  streamCallback: CopilotStreamCallback,
) {
  log.debug("Starting copilot chat request flow");
  const msaChatSession = await getAuthSession(
    [scopes.chatApi, `VSCODE_TENANT:common`, `VSCODE_CLIENT_ID:${chatApp}`],
    getRandomGuid(),
  );
  if (!msaChatSession) {
    throw Error("Failed to get MSA chat token");
  }

  await chatRequest(
    msaChatSession.accessToken,
    question,
    streamCallback,
    latexContext,
  );
}

async function chatRequest(
  token: string,
  question: string,
  streamCallback: CopilotStreamCallback,
  context?: string,
): Promise<any> {
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
        log.debug("Received copilot fetch message: ", ev);
        const messageReceived: QuantumChatResponse = JSON.parse(ev.data);
        log.debug("Received message: ", messageReceived);
        if (messageReceived.Delta) streamCallback(messageReceived.Delta, false);
      },
    });

    log.debug("ChatAPI fetch completed");
    streamCallback("", true);
    return Promise.resolve({});
  } catch (error) {
    log.error("ChatAPI fetch failed with error: ", error);
    throw error;
  }
}
