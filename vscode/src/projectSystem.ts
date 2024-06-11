// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IProjectConfig, getProjectLoader, log } from "qsharp-lang";
import * as vscode from "vscode";
import { Uri } from "vscode";
import { URI, Utils } from "vscode-uri";
import { updateQSharpJsonDiagnostics } from "./diagnostics";
import { IPackageInfo, ProjectLoader } from "../../npm/qsharp/lib/web/qsc_wasm";

/**
 * Finds and parses a manifest. Returns `null` if no manifest was found for the given uri, or if the manifest
 * was malformed.
 */
// async function getManifest(uri: string): Promise<
//   | ({
//       manifestDirectory: Uri;
//       manifestUri: Uri;
//     } & QSharpJsonManifest)
//   | null
// > {
//   const manifestDocument = await findManifestDocument(uri);
//   if (manifestDocument === null) {
//     return null;
//   }
//   let result;
//   try {
//     result = await getManifestThrowsOnParseFailure(uri);
//   } catch (e) {
//     log.warn(
//       `failed to parse manifest at ${manifestDocument.uri.toString()}`,
//       e,
//     );
//     updateQSharpJsonDiagnostics(
//       manifestDocument.uri,
//       "Failed to parse Q# manifest. For a minimal Q# project manifest, try: {}",
//     );
//     return null;
//   }
//   return result;
// }

/** Returns the manifest document if one is found
 * returns null otherwise
 */
async function findManifestDocument(
  currentDocumentUriString: string,
): Promise<{ directory: vscode.Uri; uri: vscode.Uri; content: string } | null> {
  // file://home/foo/bar/src/document.qs
  // or, e.g. in vscode on a virtual file system,
  // vscode-vfs://github%2B7b2276223a312c22726566223a7b2274797065223a332c226964223a22383439227d7d/microsoft/qsharp/samples/shor.qs
  const currentDocumentUri = URI.parse(currentDocumentUriString);

  // Untitled documents don't have a file location, thus can't have a manifest
  if (currentDocumentUri.scheme === "untitled") return null;

  // just the parent
  // e.g.
  // file://home/foo/bar/src
  let uriToQuery = Utils.dirname(currentDocumentUri);

  let attempts = 100;

  let seenSrcDir = false;

  while (attempts > 0) {
    attempts--;

    // Make sure that the path doesn't go above one of the open workspaces.
    if (
      !vscode.workspace.workspaceFolders?.some((root) =>
        uriToQuery.toString().startsWith(root.uri.toString()),
      )
    ) {
      log.debug("Aborting search for manifest file outside of workspace");
      return null;
    }

    if (seenSrcDir) {
      const listing: { uri: vscode.Uri; content: string } | null =
        await tryReadManifestInDir(uriToQuery);

      if (listing) {
        return { directory: Utils.dirname(listing.uri), ...listing };
      }
    }
    if (uriToQuery.toString().endsWith("src")) {
      seenSrcDir = true;
    } else {
      seenSrcDir = false;
    }

    const oldUriToQuery = uriToQuery;
    uriToQuery = Utils.resolvePath(uriToQuery, "..");
    if (oldUriToQuery === uriToQuery) {
      return null;
    }
  }
  return null;
}

async function tryReadManifestInDir(
  uriToQuery: URI,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  const potentialManifestLocation = Utils.joinPath(uriToQuery, "qsharp.json");

  let listing: { uri: vscode.Uri; content: string } | null = null;
  try {
    listing = await readFileUri(potentialManifestLocation);
  } catch (err) {
    log.error("Error thrown when reading file: ", err);
  }
  return listing;
}

// this function currently assumes that `directoryQuery` will be a relative path from
// the root of the workspace
export async function listDir(
  directoryQuery: string,
): Promise<[string, number][]> {
  const uriToQuery = vscode.Uri.parse(directoryQuery);

  const fileSearchResult = await vscode.workspace.fs.readDirectory(uriToQuery);
  const mappedFiles: [string, vscode.FileType][] = fileSearchResult.map(
    ([name, type]) => [Utils.joinPath(uriToQuery, name).toString(), type],
  );

  return mappedFiles;
}

export async function readFile(uri: string): Promise<string | null> {
  const file = await readFileUri(uri);
  return file?.content || null;
}

async function readFileUri(
  maybeUri: string | vscode.Uri,
): Promise<{ uri: vscode.Uri; content: string } | null> {
  const uri: vscode.Uri = (maybeUri as any).path
    ? (maybeUri as vscode.Uri)
    : vscode.Uri.parse(maybeUri as string);

  // If any open documents match this uri, return their contents instead of from disk
  const opendoc = vscode.workspace.textDocuments.find(
    (opendoc) => opendoc.uri.toString() === uri.toString(),
  );

  if (opendoc) {
    return {
      content: opendoc.getText(),
      uri,
    };
  }

  try {
    return await vscode.workspace.fs.readFile(uri).then((res) => {
      return {
        content: new TextDecoder().decode(res),
        uri: uri,
      };
    });
  } catch (_err) {
    // `readFile` should throw the below if the file is not found
    if (
      !(_err instanceof vscode.FileSystemError && _err.code === "FileNotFound")
    ) {
      log.error("Unexpected error trying to read file", _err);
    }
    return null;
  }
}

let projectLoader: ProjectLoader | undefined = undefined;

function parseManifestOrThrow(manifestDocument: {
  uri: vscode.Uri;
  content: string;
}) {
  let parsedManifest: QSharpJsonManifest | null = null;
  try {
    parsedManifest = JSON.parse(manifestDocument.content); // will throw if invalid
  } catch (e: any) {
    updateQSharpJsonDiagnostics(
      manifestDocument.uri,
      "Failed to parse Q# manifest. For a minimal Q# project manifest, try: {}",
    );
    throw new Error(
      "Failed to parse qsharp.json. For a minimal Q# project manifest, try: {}",
    );
  }

  updateQSharpJsonDiagnostics(manifestDocument.uri);

  return {
    manifestUri: manifestDocument.uri,
    ...parsedManifest,
  };
}

/**
 * Given a Q# Document URI, returns the configuration and list of complete source files
 * associated with that document.
 *
 * If there is a qsharp.json manifest for this document, the settings from that are used.
 *
 * If a manifest is not found, the returned project contains the single input file and the default settings.
 *
 * @param documentUri A Q# document.
 * @returns The project configuration for that document.
 * @throws Error if the qsharp.json cannot be parsed.
 */
export async function loadProject(
  documentUri: vscode.Uri,
): Promise<IProjectConfig> {
  const manifestDocument = await findManifestDocument(documentUri.toString());

  if (!manifestDocument) {
    // return just the one file if we are in single file mode
    return await singleFileProject(documentUri);
  }

  // Shouldn't return null because we already passed in content
  return (await loadProjectInner(manifestDocument))!;
}

export async function loadProjectNoSingleFile(
  documentUri: vscode.Uri,
): Promise<IProjectConfig | null> {
  // TODO: this is a perf fix.... sad
  await new Promise((r) => setTimeout(r, 0));

  const manifestDocument = await findManifestDocument(documentUri.toString());

  if (!manifestDocument) {
    return null;
  }

  return loadProjectInner(manifestDocument);
}

export async function loadProjectInner(manifestDocument: {
  directory: vscode.Uri;
  uri: vscode.Uri;
  content?: string;
}): Promise<IProjectConfig | null> {
  const directory = manifestDocument.directory;
  const manifestUri = manifestDocument.uri;
  const packages = {};
  const errors: string[] = [];
  const stack: PackageKey[] = [];

  const content =
    manifestDocument.content ||
    (await tryReadManifestInDir(directory))?.content;
  if (!content) {
    return null;
  }

  const root = await collectLocalPackage(stack, packages, errors, directory);

  if (errors.length > 0) {
    for (const error of errors) {
      updateQSharpJsonDiagnostics(manifestUri, error);
    }
  }

  if (!root) {
    return null;
  }

  const packageGraphSources = {
    root,
    packages,
  };

  log.info(
    `resolved package graph with sources: ${JSON.stringify(packageGraphSources, undefined, 2)}`,
  );

  // Use only the lint config from the root package
  const rootPackage = globalCache[getKeyForLocalPackage(directory)];
  const lints =
    "manifest" in rootPackage ? rootPackage.manifest.lints ?? [] : [];

  return {
    projectName: Utils.basename(directory) || "Q# Project",
    projectUri: manifestUri.toString(),
    packageGraphSources: packageGraphSources,
    lints,
  };
}

async function singleFileProject(
  documentUri: vscode.Uri,
): Promise<IProjectConfig> {
  const file = await vscode.workspace.openTextDocument(documentUri);

  return {
    projectName: Utils.basename(documentUri),
    projectUri: documentUri.toString(),
    packageGraphSources: {
      root: {
        sources: [[documentUri.toString(), file.getText()]] as [
          string,
          string,
        ][],
        languageFeatures: [],
        dependencies: {},
      },
      packages: {},
    },
    lints: [],
  };
}

async function loadPackage(
  manifestDirectory: Uri,
  manifest: QSharpJsonManifest,
): Promise<IPackageInfo> {
  if (!projectLoader) {
    projectLoader = await getProjectLoader(readFile, listDir);
  }

  const sources = await projectLoader.load_project({
    manifestDirectory: manifestDirectory.toString(),
    ...manifest,
  });
  const manifestDependencies = manifest.dependencies || {};
  const dependencies = Object.keys(manifestDependencies).reduce(
    (aliasToKey: { [alias: string]: string }, alias) => {
      aliasToKey[alias] = getKeyForDependencyDefinition(
        manifestDirectory,
        manifestDependencies[alias],
      );
      return aliasToKey;
    },
    {},
  );

  const packageInfo = {
    sources,
    languageFeatures: manifest.languageFeatures || [],
    dependencies,
  };
  return packageInfo;
}

type DependencyDefinition =
  | {
      github: {
        owner: string;
        repo: string;
        ref: string;
      };
    }
  | {
      path: string;
    };

interface QSharpJsonManifest {
  languageFeatures?: string[];
  lints?: {
    lint: string;
    level: string;
  }[];
  dependencies?: {
    [alias: string]: DependencyDefinition;
  };
}

type PackageKey = string;

async function collectLocalPackage(
  stack: PackageKey[],
  packages: Record<PackageKey, IPackageInfo>,
  errors: string[],
  directory: Uri,
): Promise<IPackageInfo | undefined> {
  const key = getKeyForLocalPackage(directory);

  const result = await readManifestAndSources(key, directory);

  if ("error" in result) {
    errors.push(result.error);
    return undefined;
  }

  const { packageInfo: pkg } = result;

  stack.push(key);

  for (const alias in pkg.dependencies) {
    const depKey = pkg.dependencies[alias];
    log.info(`adding dependency ${alias}: ${depKey} for package ${key}`);

    if (stack.includes(depKey)) {
      log.info(`circular dependency detected from ${depKey}`);
      errors.push(
        `Circular dependency detected: ${stack.join(" -> ")} -> ${depKey}`,
      );
      continue;
    }

    const dependencyDefinition = decodeDependencyDefinitionFromKey(depKey);
    if ("github" in dependencyDefinition) {
      throw new Error("support github");
    }

    let depDirectory: Uri;

    try {
      depDirectory = Uri.parse(dependencyDefinition.path, true);
    } catch (e) {
      errors.push(`Invalid path for dependency: ${dependencyDefinition.path}`);
      continue;
    }

    const depPkg = await collectLocalPackage(
      stack,
      packages,
      errors,
      depDirectory,
    );
    if (depPkg) {
      packages[depKey] = depPkg;
    }

    // TODO: absolute paths
    // TODO: os-specific slashes
  }
  stack.pop();
  return pkg;
}

const globalCache: Record<
  PackageKey,
  | {
      manifest: QSharpJsonManifest;
      packageInfo: IPackageInfo;
    }
  | {
      error: string;
    }
> = {};
async function readManifestAndSources(key: PackageKey, directory: vscode.Uri) {
  const cached = globalCache[key];
  if (cached) {
    log.info(`package ${key} already in cache`);
    return cached;
  }

  const manifestDocument = await tryReadManifestInDir(directory);
  if (!manifestDocument) {
    globalCache[key] = {
      error: `No qsharp.json found in directory ${directory.toString()}`,
    };
    return globalCache[key];
  }

  let manifest;
  try {
    manifest = parseManifestOrThrow(manifestDocument);
  } catch (e) {
    globalCache[key] = {
      error: `Could not parse qsharp.json in directory ${directory.toString()}`,
    };
    return globalCache[key];
  }
  const pkg = {
    packageInfo: await loadPackage(directory, manifest),
    manifest,
  };

  log.info(`adding package ${key} to cache`);
  globalCache[key] = pkg;
  return pkg;
}

function getKeyForDependencyDefinition(
  fromDirectory: Uri,
  dependencyDefinition: DependencyDefinition,
): string {
  if ("github" in dependencyDefinition) {
    // TODO: github
    // TODO: add some kind of limit in case someone wants to DOS the extension with 1000000 dependencies
    // TODO: github projects shouldn't contain local references
    throw new Error("GitHub dependencies are not supported yet");
  }
  const absoluteDir = Utils.resolvePath(
    fromDirectory,
    dependencyDefinition.path,
  );
  // TODO: terribly malformed paths?
  const key = getKeyForLocalPackage(absoluteDir);
  log.info(
    `getKeyForDependencyDefinition: ${fromDirectory.toString()},${JSON.stringify(dependencyDefinition)} -> ${key}`,
  );
  return key;
}

function getKeyForLocalPackage(absoluteDir: URI) {
  return absoluteDir.toString();
}

function decodeDependencyDefinitionFromKey(key: string): DependencyDefinition {
  log.info(`decodeDependencyDefinitionFromKey: ${key} -> { path: ${key} }`);
  // TODO: support github keys
  return { path: key };
}
