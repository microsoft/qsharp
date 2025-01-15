// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { CopilotEvent } from "../commonTypes";

export type CopilotEventHandler = (event: CopilotEvent) => void;

export interface ICopilotFactory {
  createCopilot(callback: CopilotEventHandler): Promise<ICopilot>;
}

export interface ICopilot {
  converse(question: string): Promise<void>;
}
