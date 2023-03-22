// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

const isNode = !!(process && process.env)

// Note: Consumers should directly import 'qsharp/browser' or 'qsharp/node', not this
// module. This is here mostly to ensure the others get linked into the top-level build.

if (isNode) {
    const mod = await import("./node.js");
} else {
    const mod = await import("./browser.js");
}
