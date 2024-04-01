// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { createWorker } from "../workers/node.js";
import { languageServiceProtocol } from "./language-service.js";

createWorker(languageServiceProtocol);
