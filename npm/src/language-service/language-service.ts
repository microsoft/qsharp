// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import type {
  IDiagnostic,
  ICompletionList,
  IHover,
  IDefinition,
  LanguageService,
} from "../../lib/node/qsc_wasm.cjs";
import { log } from "../log.js";
import {
  VSDiagnostic,
  mapDiagnostics,
  mapUtf16UnitsToUtf8Units,
  mapUtf8UnitsToUtf16Units,
} from "../vsdiagnostic.js";
import { IServiceEventTarget, IServiceProxy } from "../worker-proxy.js";
type QscWasm = typeof import("../../lib/node/qsc_wasm.cjs");

// Only one event type for now
export type LanguageServiceEvent = {
  type: "diagnostics";
  detail: {
    uri: string;
    version: number;
    diagnostics: VSDiagnostic[];
  };
};

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ILanguageService {
  updateDocument(
    uri: string,
    version: number,
    code: string,
    isExe: boolean
  ): Promise<void>;
  closeDocument(uri: string): Promise<void>;
  getCompletions(documentUri: string, offset: number): Promise<ICompletionList>;
  getHover(documentUri: string, offset: number): Promise<IHover | null>;
  getDefinition(
    documentUri: string,
    offset: number
  ): Promise<IDefinition | null>;
  dispose(): Promise<void>;

  addEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void
  ): void;

  removeEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void
  ): void;
}

export const qsharpLibraryUriScheme = "qsharp-library-source";

export type ILanguageServiceWorker = ILanguageService & IServiceProxy;

export class QSharpLanguageService implements ILanguageService {
  private languageService: LanguageService;
  private eventHandler =
    new EventTarget() as IServiceEventTarget<LanguageServiceEvent>;

  // We need to keep a copy of the code for mapping diagnostics to utf16 offsets
  private code: { [uri: string]: string } = {};

  constructor(wasm: QscWasm) {
    log.info("Constructing a QSharpLanguageService instance");
    this.languageService = new wasm.LanguageService(
      this.onDiagnostics.bind(this)
    );
  }

  async updateDocument(
    documentUri: string,
    version: number,
    code: string,
    isExe: boolean
  ): Promise<void> {
    this.code[documentUri] = code;
    this.languageService.update_document(documentUri, version, code, isExe);
  }

  async closeDocument(documentUri: string): Promise<void> {
    delete this.code[documentUri];
    this.languageService.close_document(documentUri);
  }

  async getCompletions(
    documentUri: string,
    offset: number
  ): Promise<ICompletionList> {
    const code = this.code[documentUri];
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_completions(
      documentUri,
      convertedOffset
    ) as ICompletionList;
    result.items.forEach((item) =>
      item.additionalTextEdits?.forEach((edit) => {
        const mappedSpan = mapUtf8UnitsToUtf16Units(
          [edit.range.start, edit.range.end],
          code
        );
        edit.range.start = mappedSpan[edit.range.start];
        edit.range.end = mappedSpan[edit.range.end];
      })
    );
    return result;
  }

  async getHover(documentUri: string, offset: number): Promise<IHover | null> {
    const code = this.code[documentUri];
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_hover(
      documentUri,
      convertedOffset
    ) as IHover | null;
    if (result) {
      const mappedSpan = mapUtf8UnitsToUtf16Units(
        [result.span.start, result.span.end],
        code
      );
      result.span.start = mappedSpan[result.span.start];
      result.span.end = mappedSpan[result.span.end];
    }
    return result;
  }

  async getDefinition(
    documentUri: string,
    offset: number
  ): Promise<IDefinition | null> {
    let code = this.code[documentUri];
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_definition(
      documentUri,
      convertedOffset
    ) as IDefinition | null;
    if (result) {
      // Inspect the URL protocol (equivalent to the URI scheme + ":").
      // If the scheme is our library scheme, we need to call the wasm to
      // provide the library file's contents to do the utf8->utf16 mapping.
      const url = new URL(result.source);
      if (url.protocol === qsharpLibraryUriScheme + ":") {
        code = wasm.get_library_source_content(url.pathname);
      }
      result.offset = mapUtf8UnitsToUtf16Units([result.offset], code)[
        result.offset
      ];
    }
    return result;
  }

  async dispose() {
    this.languageService.free();
  }

  addEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void
  ) {
    this.eventHandler.addEventListener(type, listener);
  }

  removeEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void
  ) {
    this.eventHandler.removeEventListener(type, listener);
  }

  onDiagnostics(uri: string, version: number, diagnostics: IDiagnostic[]) {
    try {
      const code = this.code[uri];
      const event = new Event("diagnostics") as LanguageServiceEvent & Event;
      event.detail = {
        uri,
        version,
        diagnostics: mapDiagnostics(diagnostics, code),
      };
      this.eventHandler.dispatchEvent(event);
    } catch (e) {
      log.error("Error in onDiagnostics", e);
    }
  }
}
