// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { messageHandler } from "qsharp-lang/debug-service-worker";

self.onmessage = messageHandler;
