// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { messageHandler } from "qsharp/debug-service-worker";

self.onmessage = messageHandler;
