// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { createWorker } from "../workers/browser.js";
import { compilerProtocol } from "./compiler.js";

// This export should be assigned to 'self.onmessage' in a WebWorker
export const messageHandler = createWorker(compilerProtocol);
