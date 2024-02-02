// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { createWorker } from "../workers/browser.js";
import { debugServiceProtocol } from "./debug-service.js";

// This export should be assigned to 'self.onmessage' in a WebWorker
export const messageHandler = createWorker(debugServiceProtocol);
