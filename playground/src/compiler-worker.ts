// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { messageHandler } from "qsharp/compiler-worker";

self.onmessage = messageHandler;
