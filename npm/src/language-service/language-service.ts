// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import type {
  IDiagnostic,
  ICompletionList,
  IHover,
  IDefinition,
  ISignatureHelp,
  LanguageService,
  IWorkspaceConfiguration,
  IWorkspaceEdit,
  ITextEdit,
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
  updateConfiguration(config: IWorkspaceConfiguration): Promise<void>;
  updateDocument(uri: string, version: number, code: string): Promise<void>;
  closeDocument(uri: string): Promise<void>;

  getCompletions(documentUri: string, offset: number): Promise<ICompletionList>;
  getHover(documentUri: string, offset: number): Promise<IHover | undefined>;
  getDefinition(
    documentUri: string,
    offset: number
  ): Promise<IDefinition | undefined>;
  getSignatureHelp(
    documentUri: string,
    offset: number
  ): Promise<ISignatureHelp | undefined>;
  getRename(
    documentUri: string,
    offset: number,
    newName: string
  ): Promise<IWorkspaceEdit | undefined>;

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
  private code: { [uri: string]: string | undefined } = {};

  constructor(wasm: QscWasm) {
    log.info("Constructing a QSharpLanguageService instance");
    this.languageService = new wasm.LanguageService(
      this.onDiagnostics.bind(this)
    );
  }

  async updateConfiguration(config: IWorkspaceConfiguration): Promise<void> {
    this.languageService.update_configuration(config);
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

  async getCompletions(
    documentUri: string,
    offset: number
  ): Promise<ICompletionList> {
    const code = this.code[documentUri];
    if (code === undefined) {
      log.error(
        `getCompletions: expected ${documentUri} to be in the document map`
      );
      return { items: [] };
    }
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_completions(
      documentUri,
      convertedOffset
    );
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

  async getHover(
    documentUri: string,
    offset: number
  ): Promise<IHover | undefined> {
    const code = this.code[documentUri];
    if (code === undefined) {
      log.error(`getHover: expected ${documentUri} to be in the document map`);
      return undefined;
    }
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_hover(documentUri, convertedOffset);
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
  ): Promise<IDefinition | undefined> {
    let code = this.code[documentUri];
    if (code === undefined) {
      log.error(
        `getDefinition: expected ${documentUri} to be in the document map`
      );
      return undefined;
    }
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_definition(
      documentUri,
      convertedOffset
    );
    if (result) {
      // Inspect the URL protocol (equivalent to the URI scheme + ":").
      // If the scheme is our library scheme, we need to call the wasm to
      // provide the library file's contents to do the utf8->utf16 mapping.
      const url = new URL(result.source);
      if (url.protocol === qsharpLibraryUriScheme + ":") {
        code = wasm.get_library_source_content(url.pathname);
        if (code === undefined) {
          log.error(`getDefinition: expected ${url} to be in the library`);
          return undefined;
        }
      }
      result.offset = mapUtf8UnitsToUtf16Units([result.offset], code)[
        result.offset
      ];
    }
    return result;
  }

  async getSignatureHelp(
    documentUri: string,
    offset: number
  ): Promise<ISignatureHelp | undefined> {
    const code = this.code[documentUri];
    if (code === undefined) {
      log.error(`expected ${documentUri} to be in the document map`);
      return undefined;
    }
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_signature_help(
      documentUri,
      convertedOffset
    );
    if (result) {
      result.signatures = result.signatures.map((sig) => {
        sig.parameters = sig.parameters.map((param) => {
          const mappedSpan = mapUtf8UnitsToUtf16Units(
            [param.label.start, param.label.end],
            sig.label
          );
          param.label.start = mappedSpan[param.label.start];
          param.label.end = mappedSpan[param.label.end];
          return param;
        });
        return sig;
      });
    }
    return result;
  }

  async getRename(
    documentUri: string,
    offset: number,
    newName: string
  ): Promise<IWorkspaceEdit | undefined> {
    const code = this.code[documentUri];
    if (code === undefined) {
      log.error(`expected ${documentUri} to be in the document map`);
      return undefined;
    }
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_rename(
      documentUri,
      convertedOffset,
      newName
    );

    const mappedChanges: [string, ITextEdit[]][] = [];
    for (const [uri, edits] of result.changes) {
      const code = this.code[uri];
      if (code) {
        const mappedEdits = edits.map((edit) => {
          const mappedSpan = mapUtf8UnitsToUtf16Units(
            [edit.range.start, edit.range.end],
            code
          );
          edit.range.start = mappedSpan[edit.range.start];
          edit.range.end = mappedSpan[edit.range.end];
          return edit;
        });
        mappedChanges.push([uri, mappedEdits]);
      }
    }
    result.changes = mappedChanges;
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
      const empty = diagnostics.length === 0;
      if (code === undefined && !empty) {
        // We need the contents of the document to convert error offsets to utf16.
        // But the contents aren't available after a document is closed.
        // It is possible to get a diagnostics event after a document is closed,
        // but it will be done with an empty array, to clear the diagnostics.
        // In that case, it's ok not to have the document contents available,
        // because there are no offsets to convert.
        log.error(`onDiagnostics: expected ${uri} to be in the document map`);
        return;
      }
      const event = new Event("diagnostics") as LanguageServiceEvent & Event;
      event.detail = {
        uri,
        version,
        diagnostics: empty ? [] : mapDiagnostics(diagnostics, code as string),
      };
      this.eventHandler.dispatchEvent(event);
    } catch (e) {
      log.error("Error in onDiagnostics", e);
    }
  }
}
