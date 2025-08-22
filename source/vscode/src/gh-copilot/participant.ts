// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";

const participantId = "chat-sample.cat";

export function registerChatParticipant(context: vscode.ExtensionContext) {
  log.debug("registering chat participant");
  const participant = vscode.chat.createChatParticipant(
    participantId,
    async function handle(
      request,
      context,
      response,
    ): Promise<vscode.ChatResult> {
      log.debug("got request");
      // const prompt = request.prompt;
      // const command = request.command;
      // const model = request.model;
      // const references = request.references;
      // const toolToken = request.toolInvocationToken;
      // const toolReferences = request.toolReferences;
      // const history = context.history;

      // response.markdown(`prompt: ${prompt}\n`);
      // response.markdown(`command: ${command}\n`);
      // response.markdown(`model: ${model.name}\n`);
      // response.markdown(
      //   `references: ${references.map((r) => r.id.toString()).join(", ")}\n`,
      // );
      // response.markdown(`toolToken: ${toolToken}\n`);
      // response.markdown(
      //   `toolReferences: ${toolReferences.map((r) => r.name.toString()).join(", ")}\n`,
      // );
      // response.markdown(
      //   `history: ${history.map((h) => h.command?.toString()).join(", ")}\n`,
      // );
      response.markdown("hi");

      return { metadata: { command: "" } };
    },
  );
  // participant.followupProvider = {
  //   provideFollowups(result, context, token) {
  //     void result;
  //     void context;
  //     void token;
  //     log.debug("providing followups");
  //     return [
  //       {
  //         prompt: "run the thing",
  //         label: "simulate",
  //         command: "run",
  //       } satisfies vscode.ChatFollowup,
  //     ];
  //   },
  // };
  context.subscriptions.push(participant);
  log.debug("chat participant registered");
}
