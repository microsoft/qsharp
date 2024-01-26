// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {
  ICompletionList,
  IHover,
  ILocation,
  ISignatureHelp,
  INotebookMetadata,
  IWorkspaceConfiguration,
  IWorkspaceEdit,
  ITextEdit,
  LanguageService,
  IPosition,
  VSDiagnostic,
} from "../../lib/node/qsc_wasm.cjs";
import { log } from "../log.js";
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
  updateNotebookDocument(
    notebookUri: string,
    version: number,
    metadata: INotebookMetadata,
    cells: {
      uri: string;
      version: number;
      code: string;
    }[],
  ): Promise<void>;
  closeDocument(uri: string): Promise<void>;
  closeNotebookDocument(notebookUri: string): Promise<void>;
  getCompletions(
    documentUri: string,
    position: IPosition,
  ): Promise<ICompletionList>;
  getHover(
    documentUri: string,
    position: IPosition,
  ): Promise<IHover | undefined>;
  getDefinition(
    documentUri: string,
    position: IPosition,
  ): Promise<ILocation | undefined>;
  getReferences(
    documentUri: string,
    position: IPosition,
    includeDeclaration: boolean,
  ): Promise<ILocation[]>;
  getSignatureHelp(
    documentUri: string,
    position: IPosition,
  ): Promise<ISignatureHelp | undefined>;
  getRename(
    documentUri: string,
    position: IPosition,
    newName: string,
  ): Promise<IWorkspaceEdit | undefined>;
  prepareRename(
    documentUri: string,
    position: IPosition,
  ): Promise<ITextEdit | undefined>;

  dispose(): Promise<void>;

  addEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void,
  ): void;

  removeEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void,
  ): void;
}

export const qsharpLibraryUriScheme = "qsharp-library-source";

export type ILanguageServiceWorker = ILanguageService & IServiceProxy;

export class QSharpLanguageService implements ILanguageService {
  private languageService: LanguageService;
  private eventHandler =
    new EventTarget() as IServiceEventTarget<LanguageServiceEvent>;

  private readFile: (uri: string) => Promise<string | null>;

  private backgroundWork: Promise<void>;

  constructor(
    wasm: QscWasm,
    readFile: (uri: string) => Promise<string | null> = () =>
      Promise.resolve(null),
    listDir: (uri: string) => Promise<[string, number][]> = () =>
      Promise.resolve([]),
    getManifest: (uri: string) => Promise<{
      manifestDirectory: string;
    } | null> = () => Promise.resolve(null),
  ) {
    log.info("Constructing a QSharpLanguageService instance");
    this.languageService = new wasm.LanguageService();

    this.backgroundWork = this.languageService.start_background_work(
      this.onDiagnostics.bind(this),
      readFile,
      listDir,
      getManifest,
    );

    this.readFile = readFile;
  }

  async updateConfiguration(config: IWorkspaceConfiguration): Promise<void> {
    this.languageService.update_configuration(config);
  }

  async updateDocument(
    documentUri: string,
    version: number,
    code: string,
  ): Promise<void> {
    this.languageService.update_document(documentUri, version, code);
  }

  async updateNotebookDocument(
    notebookUri: string,
    version: number,
    metadata: INotebookMetadata,
    cells: { uri: string; version: number; code: string }[],
  ): Promise<void> {
    this.languageService.update_notebook_document(notebookUri, metadata, cells);
  }

  async closeDocument(documentUri: string): Promise<void> {
    this.languageService.close_document(documentUri);
  }

  async closeNotebookDocument(documentUri: string): Promise<void> {
    this.languageService.close_notebook_document(documentUri);
  }

  async getCompletions(
    documentUri: string,
    position: IPosition,
  ): Promise<ICompletionList> {
    return this.languageService.get_completions(documentUri, position);
  }

  async getHover(
    documentUri: string,
    position: IPosition,
  ): Promise<IHover | undefined> {
    return this.languageService.get_hover(documentUri, position);
  }

  async getDefinition(
    documentUri: string,
    position: IPosition,
  ): Promise<ILocation | undefined> {
    return this.languageService.get_definition(documentUri, position);
  }

  async getReferences(
    documentUri: string,
    position: IPosition,
    includeDeclaration: boolean,
  ): Promise<ILocation[]> {
    return this.languageService.get_references(
      documentUri,
      position,
      includeDeclaration,
    );
  }

  async getSignatureHelp(
    documentUri: string,
    position: IPosition,
  ): Promise<ISignatureHelp | undefined> {
    return this.languageService.get_signature_help(documentUri, position);
  }

  async getRename(
    documentUri: string,
    position: IPosition,
    newName: string,
  ): Promise<IWorkspaceEdit | undefined> {
    return this.languageService.get_rename(documentUri, position, newName);
  }

  async prepareRename(
    documentUri: string,
    position: IPosition,
  ): Promise<ITextEdit | undefined> {
    return this.languageService.prepare_rename(documentUri, position);
  }

  async dispose() {
    this.languageService.stop_background_work();
    await this.backgroundWork;
    this.languageService.free();
  }

  addEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void,
  ) {
    this.eventHandler.addEventListener(type, listener);
  }

  removeEventListener<T extends LanguageServiceEvent["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvent, { type: T }>) => void,
  ) {
    this.eventHandler.removeEventListener(type, listener);
  }

  async onDiagnostics(
    uri: string,
    version: number | undefined,
    diagnostics: VSDiagnostic[],
  ) {
    try {
      const event = new Event("diagnostics") as LanguageServiceEvent & Event;
      event.detail = {
        uri,
        version: version ?? 0,
        diagnostics,
      };
      this.eventHandler.dispatchEvent(event);
    } catch (e) {
      log.error("Error in onDiagnostics", e);
    }
  }
}
