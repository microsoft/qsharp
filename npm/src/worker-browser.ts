// This module supports running the compiler inside a browser WebWorker. This is set as
// the "qsharp/worker" entry point using 'conditional exports' in package.json.
// The worker script to be loaded in the browser should import the handler via
// `import { compilerMessageHandler } from "qsharp/worker"` and assign this to 'self.onmessage'.

// This export should be assigned to 'self.onmessage' in a WebWorker
export { messageHandler as compilerMessageHandler } from "./compiler/worker-browser.js";
