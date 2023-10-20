// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log, samples } from "qsharp-lang";
import * as vscode from "vscode";

export const scheme = "qsharp-vfs";

function populateSamples(vfs: MemFS) {
  const rootDir = vscode.Uri.parse(`${scheme}:/`);
  if (
    // Don't overwrite (or recreate) any existing samples
    !vfs.readDirectory(rootDir).includes(["samples", vscode.FileType.Directory])
  ) {
    const encoder = new TextEncoder();
    vfs.createDirectory(vscode.Uri.parse(`${scheme}:/samples`));

    samples.forEach((sample) => {
      vfs.writeFile(
        vscode.Uri.parse(`${scheme}:/samples/${sample.title}.qs`),
        encoder.encode(sample.code),
        { create: true, overwrite: true }
      );
    });
  }
}

export async function initFileSystem(context: vscode.ExtensionContext) {
  const vfs = new MemFS();
  populateSamples(vfs);

  context.subscriptions.push(
    vscode.workspace.registerFileSystemProvider(scheme, vfs, {
      isCaseSensitive: true,
      isReadonly: false,
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.openSamples", async () => {
      await vscode.commands.executeCommand(
        "vscode.openFolder",
        vscode.Uri.parse(`${scheme}:/samples`),
        { forceNewWindow: false, forceReuseWindow: true }
      );
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.webOpener", async (uri) => {
      log.info("typeof uri: " + typeof uri);
      log.info(`webOpener with URI ${uri}`);

      if (typeof uri === "string") {
        const uriObj = vscode.Uri.parse(uri);
        const codeFile = vscode.Uri.parse(`${scheme}:/code.qs`);
        const encoder = new TextEncoder();
        vfs.writeFile(codeFile, encoder.encode(uriObj.fragment), {
          create: true,
          overwrite: true,
        });
        await vscode.commands.executeCommand("vscode.open", codeFile);
      }
    })
  );
}

// basename and dirname are only xcalled with a vscode.uri 'path', which should be a well-formed posix path
// Below tested to align with how NodeJS path.basename and path.dirname work
function basename(path: string) {
  path = path.replace(/\/+$/, "");
  if (!path) {
    return "";
  }
  return path.substring(path.lastIndexOf("/") + 1);
}

function dirname(path: string) {
  if (!path) return ".";
  if (path === "/") return "/";

  path = path.replace(/\/+$/, "");
  const offset = path.lastIndexOf("/");

  switch (offset) {
    case -1:
      return ".";
    case 0:
      return "/";
    default:
      return path.substring(0, offset);
  }
}

// The below largely taken from the reference implementation at
// https://github.com/microsoft/vscode-extension-samples/blob/main/fsprovider-sample/src/fileSystemProvider.ts
export class File implements vscode.FileStat {
  type: vscode.FileType;
  ctime: number;
  mtime: number;
  size: number;

  name: string;
  data?: Uint8Array;

  constructor(name: string) {
    this.type = vscode.FileType.File;
    this.ctime = Date.now();
    this.mtime = Date.now();
    this.size = 0;
    this.name = name;
  }
}

export class Directory implements vscode.FileStat {
  type: vscode.FileType;
  ctime: number;
  mtime: number;
  size: number;

  name: string;
  entries: Map<string, File | Directory>;

  constructor(name: string) {
    this.type = vscode.FileType.Directory;
    this.ctime = Date.now();
    this.mtime = Date.now();
    this.size = 0;
    this.name = name;
    this.entries = new Map();
  }
}

export type Entry = File | Directory;

export class MemFS implements vscode.FileSystemProvider {
  root = new Directory(``);

  stat(uri: vscode.Uri): vscode.FileStat {
    log.debug(`stat: ${uri.path}`);
    return this._lookup(uri, false);
  }

  readDirectory(uri: vscode.Uri): [string, vscode.FileType][] {
    log.debug(`readDirectory: ${uri.path}`);
    const entry = this._lookupAsDirectory(uri, false);
    const result: [string, vscode.FileType][] = [];
    for (const [name, child] of entry.entries) {
      result.push([name, child.type]);
    }
    return result;
  }

  readFile(uri: vscode.Uri): Uint8Array {
    log.debug("readFile: " + uri.path);
    const data = this._lookupAsFile(uri, false).data;
    if (data) {
      return data;
    }
    throw vscode.FileSystemError.FileNotFound();
  }

  writeFile(
    uri: vscode.Uri,
    content: Uint8Array,
    options: { create: boolean; overwrite: boolean }
  ): void {
    log.debug("writeFile: " + uri.path);

    const baseName = basename(uri.path);
    const parent = this._lookupParentDirectory(uri);
    let entry = parent.entries.get(baseName);
    if (entry instanceof Directory) {
      throw vscode.FileSystemError.FileIsADirectory(uri);
    }
    if (!entry && !options.create) {
      throw vscode.FileSystemError.FileNotFound(uri);
    }
    if (entry && options.create && !options.overwrite) {
      throw vscode.FileSystemError.FileExists(uri);
    }
    if (!entry) {
      entry = new File(baseName);
      parent.entries.set(baseName, entry);
      this._fireSoon({ type: vscode.FileChangeType.Created, uri });
    }
    entry.mtime = Date.now();
    entry.size = content.byteLength;
    entry.data = content;

    this._fireSoon({ type: vscode.FileChangeType.Changed, uri });
  }

  rename(
    oldUri: vscode.Uri,
    newUri: vscode.Uri,
    options: { overwrite: boolean }
  ): void {
    if (!options.overwrite && this._lookup(newUri, true)) {
      throw vscode.FileSystemError.FileExists(newUri);
    }

    const entry = this._lookup(oldUri, false);
    const oldParent = this._lookupParentDirectory(oldUri);

    const newParent = this._lookupParentDirectory(newUri);
    const newName = basename(newUri.path);

    oldParent.entries.delete(entry.name);
    entry.name = newName;
    newParent.entries.set(newName, entry);

    this._fireSoon(
      { type: vscode.FileChangeType.Deleted, uri: oldUri },
      { type: vscode.FileChangeType.Created, uri: newUri }
    );
  }

  delete(uri: vscode.Uri): void {
    const dirName = uri.with({ path: dirname(uri.path) });
    const baseName = basename(uri.path);
    const parent = this._lookupAsDirectory(dirName, false);
    if (!parent.entries.has(baseName)) {
      throw vscode.FileSystemError.FileNotFound(uri);
    }
    parent.entries.delete(baseName);
    parent.mtime = Date.now();
    parent.size -= 1;
    this._fireSoon(
      { type: vscode.FileChangeType.Changed, uri: dirName },
      { uri, type: vscode.FileChangeType.Deleted }
    );
  }

  createDirectory(uri: vscode.Uri): void {
    const baseName = basename(uri.path);
    const dirName = uri.with({ path: dirname(uri.path) });
    const parent = this._lookupAsDirectory(dirName, false);

    const entry = new Directory(baseName);
    parent.entries.set(entry.name, entry);
    parent.mtime = Date.now();
    parent.size += 1;
    this._fireSoon(
      { type: vscode.FileChangeType.Changed, uri: dirName },
      { type: vscode.FileChangeType.Created, uri }
    );
  }

  private _lookup(uri: vscode.Uri, silent: false): Entry;
  private _lookup(uri: vscode.Uri, silent: boolean): Entry | undefined;
  private _lookup(uri: vscode.Uri, silent: boolean): Entry | undefined {
    const parts = uri.path.split("/");
    let entry: Entry = this.root;
    for (const part of parts) {
      if (!part) {
        continue;
      }
      let child: Entry | undefined;
      if (entry instanceof Directory) {
        child = entry.entries.get(part);
      }
      if (!child) {
        if (!silent) {
          throw vscode.FileSystemError.FileNotFound(uri);
        } else {
          return undefined;
        }
      }
      entry = child;
    }
    return entry;
  }

  private _lookupAsDirectory(uri: vscode.Uri, silent: boolean): Directory {
    const entry = this._lookup(uri, silent);
    if (entry instanceof Directory) {
      return entry;
    }
    throw vscode.FileSystemError.FileNotADirectory(uri);
  }

  private _lookupAsFile(uri: vscode.Uri, silent: boolean): File {
    const entry = this._lookup(uri, silent);
    if (entry instanceof File) {
      return entry;
    }
    throw vscode.FileSystemError.FileIsADirectory(uri);
  }

  private _lookupParentDirectory(uri: vscode.Uri): Directory {
    const dirName = uri.with({ path: dirname(uri.path) });
    return this._lookupAsDirectory(dirName, false);
  }

  private _emitter = new vscode.EventEmitter<vscode.FileChangeEvent[]>();
  private _bufferedEvents: vscode.FileChangeEvent[] = [];
  private _fireSoonHandle?: any;

  readonly onDidChangeFile: vscode.Event<vscode.FileChangeEvent[]> =
    this._emitter.event;

  watch(): vscode.Disposable {
    // TODO: Docs say this shouldn't fire for excluded files, but it does...
    // ignore, fires for all changes...
    return new vscode.Disposable(() => {
      return;
    });
  }

  private _fireSoon(...events: vscode.FileChangeEvent[]): void {
    this._bufferedEvents.push(...events);

    if (this._fireSoonHandle) {
      clearTimeout(this._fireSoonHandle);
    }

    this._fireSoonHandle = setTimeout(() => {
      this._emitter.fire(this._bufferedEvents);
      this._bufferedEvents.length = 0;
    }, 5);
  }
}
