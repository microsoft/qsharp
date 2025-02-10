// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";

export type ServiceConfig = {
  /** Friendly name. */
  name: string;
  /** Choose this service by default. */
  default?: boolean;
} & (
  | ({
      /** API type */
      type: "AzureQuantum";
    } & AzureQuantumServiceConfig)
  | ({
      /** API type */
      type: "OpenAI";
    } & OpenAIServiceConfig)
);

export type AzureQuantumServiceConfig = {
  /** The URL for the chat API */
  chatEndpointUrl: string;
  /** Whether to perform MSA auth or use a dummy token */
  msaAuth: boolean;
};

export type OpenAIServiceConfig = {
  /** OpenAI API key */
  apiKey: string;
  /** System prompt */
  systemPrompt: string;
};

export function getServices(): ServiceConfig[] {
  const services = vscode.workspace
    .getConfiguration("Q#")
    .get<ServiceConfig[]>("chat.services", []);
  if (services.length === 0) {
    log.info("No chat services configured.");
  }

  return services;
}

export function getDefaultConfig(): ServiceConfig {
  const services = getServices();
  const config = services.find((s) => s.default) ?? services[0];
  if (!config) {
    log.error('No services configured in "Q#.chat.services" setting');
    throw new Error('No services configured in "Q#.chat.services" setting');
  }
  return config;
}
