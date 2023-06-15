import { compilerMessageHandler } from "qsharp/worker";

self.onmessage = compilerMessageHandler;
