// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {
  IDiagnostic,
  ICompletionList,
  IHover,
  IDefinition,
  LanguageService,
} from "../../lib/node/qsc_wasm.cjs";
import { log } from "../log.js";
import {
  mapDiagnostics,
  mapUtf16UnitsToUtf8Units,
  mapUtf8UnitsToUtf16Units,
} from "../vsdiagnostic.js";
import { ILanguageServiceEventTarget, makeEvent } from "./events.js";

// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ILanguageService {
  updateDocument(uri: string, version: number, code: string): Promise<void>;
  closeDocument(uri: string): Promise<void>;
  getCompletions(documentUri: string, offset: number): Promise<ICompletionList>;
  getHover(documentUri: string, offset: number): Promise<IHover | null>;
  getDefinition(
    documentUri: string,
    offset: number
  ): Promise<IDefinition | null>;
}

export class QSharpLanguageService implements ILanguageService {
  private wasm: Wasm;
  private eventHandler: ILanguageServiceEventTarget;
  private languageService: LanguageService;

  // We need to keep a copy of the code for mapping diagnostics
  // It would be much better if the wasm layer could do the utf16 mapping
  // but here we are
  private code: { [uri: string]: string } = {};

  constructor(wasm: Wasm, eventHandler: ILanguageServiceEventTarget) {
    log.info("Constructing a QSharpLanguageService instance");
    this.wasm = wasm;
    this.eventHandler = eventHandler;
    this.languageService = new this.wasm.LanguageService(
      this.onDiagnostics.bind(this)
    );
    // TODO: do we call free() on this at some point?
  }

  async updateDocument(
    documentUri: string,
    version: number,
    code: string
  ): Promise<void> {
    this.code[documentUri] = code;
    this.languageService.update_document(documentUri, version, code);
  }

  async closeDocument(documentUri: string): Promise<void> {
    delete this.code[documentUri];
    this.languageService.close_document(documentUri);
  }

  onDiagnostics(uri: string, version: number, diagnostics: IDiagnostic[]) {
    try {
      const code = this.code[uri];
      this.eventHandler.dispatchEvent(
        makeEvent("diagnostics", {
          uri,
          version,
          diagnostics: mapDiagnostics(diagnostics, code),
        })
      );
    } catch (e) {
      log.error("Error in onDiagnostics", e);
    }
  }

  async getCompletions(
    documentUri: string,
    offset: number
  ): Promise<ICompletionList> {
    const code = this.code[documentUri];
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    return this.languageService.get_completions(documentUri, convertedOffset);
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
      result.span.start = mappedSpan[0];
      result.span.end = mappedSpan[1];
    }
    return result;
  }

  async getDefinition(
    documentUri: string,
    offset: number
  ): Promise<IDefinition | null> {
    const code = this.code[documentUri];
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_definition(
      documentUri,
      convertedOffset
    ) as IDefinition | null;
    if (result) {
      result.offset = mapUtf8UnitsToUtf16Units([result.offset], code)[
        result.offset
      ];
    }
    return result;
  }
}
