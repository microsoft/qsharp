// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../lib/web/qsc_wasm.js";
type QscWasm = typeof import("../lib/web/qsc_wasm.js");

import { callAndTransformExceptions } from "./diagnostics.js";

export class ProjectLoader {
  private nativeLoader: wasm.ProjectLoader;

  constructor(wasm: QscWasm, host: wasm.IProjectHost) {
    this.nativeLoader = new wasm.ProjectLoader(host);
  }

  dispose() {
    this.nativeLoader.free();
  }

  loadQSharpProject(directory: string): Promise<wasm.IProjectConfig> {
    return callAndTransformExceptions(() =>
      this.nativeLoader.load_project_with_deps(directory),
    );
  }

  loadOpenQasmProject(document: string): Promise<wasm.IProjectConfig> {
    return callAndTransformExceptions(() =>
      this.nativeLoader.load_openqasm_project(document),
    );
  }

  async getEntryProfile(
    fileName: string,
    source: string,
  ): Promise<wasm.TargetProfile | undefined> {
    return callAndTransformExceptions(
      async () =>
        wasm.ProjectLoader.get_entry_profile(fileName, source) as
          | wasm.TargetProfile
          | undefined,
    );
  }
}
