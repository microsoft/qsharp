// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IProjectConfig, getProjectLoader, log } from "qsharp-lang";
import * as vscode from "vscode";
import { Uri } from "vscode";
import { URI, Utils } from "vscode-uri";
import { updateQSharpJsonDiagnostics } from "./diagnostics";
import {
  IPackageInfo,
  PackageKey,
  ProjectLoader,
} from "../../npm/qsharp/lib/web/qsc_wasm";

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

  const root = await collectPackage(stack, packages, errors, {
    path: directory.toString(),
  });

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

  log.debug(
    `resolved package graph with sources: ${JSON.stringify(packageGraphSources, undefined, 2)}`,
  );

  // Use only the lint config from the root package
  const rootPackage =
    globalCache[getKeyForDependencyDefinition({ path: directory.toString() })];
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

// TODO: we should be robust against manifests missing some of these elements
type DependencyDefinition =
  | {
      github: {
        owner: string;
        repo: string;
        ref: string;
        path?: string;
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

async function collectPackage(
  stack: PackageKey[],
  packages: Record<PackageKey, IPackageInfo>,
  errors: string[],
  thisPkg: DependencyDefinition,
): Promise<IPackageInfo | undefined> {
  const key = getKeyForDependencyDefinition(thisPkg);
  const result = await readManifestAndSources(key, thisPkg);

  if ("error" in result) {
    errors.push(result.error);
    return undefined;
  }

  const { packageInfo: pkg } = result;

  stack.push(key);

  for (const alias in pkg.dependencies) {
    const depKey = pkg.dependencies[alias];
    log.trace(`adding dependency ${alias}: ${depKey} for package ${key}`);

    if (stack.includes(depKey)) {
      // TODO: ok to disallow circular dependencies?
      // Technically we could support them but it's a pain
      log.info(`circular dependency detected from ${depKey}`);
      errors.push(
        `Circular dependency detected: ${stack.join(" -> ")} -> ${depKey}`,
      );
      continue;
    }

    const dependencyDefinition = decodeDependencyDefinitionFromKey(depKey);
    if ("path" in dependencyDefinition && "github" in thisPkg) {
      errors.push(
        `Local dependencies not supported for github dependencies - package ${thisPkg}, dependency ${alias}`,
      );
      continue;
    }

    const depPkg = await collectPackage(
      stack,
      packages,
      errors,
      dependencyDefinition,
    );

    if (depPkg) {
      // TODO: do we end up adding the same package twice??
      packages[depKey] = depPkg;
    }

    // TODO: absolute paths
    // TODO: os-specific slashes
  }
  stack.pop();
  return pkg;
}

// TODO: what's the point of caching local deps? when do we invalidate?
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
async function readManifestAndSources(
  key: PackageKey,
  thisPkg: DependencyDefinition,
): Promise<
  | { manifest: QSharpJsonManifest; packageInfo: IPackageInfo }
  | { error: string }
> {
  const cached = globalCache[key];
  if (cached) {
    log.trace(`package ${key} already in cache`);
    return cached;
  }

  const result =
    "github" in thisPkg
      ? await readGithubManifestAndSources(thisPkg.github)
      : await readLocalManifestAndSources(thisPkg.path);

  if ("error" in result) {
    globalCache[key] = {
      error: result.error,
    };
    return globalCache[key];
  }
  const { manifestContent, sources } = result;

  let manifest: QSharpJsonManifest;
  try {
    manifest = JSON.parse(manifestContent);
  } catch (e) {
    globalCache[key] = {
      error: `Failed to parse qsharp.json for dependency ${key}: ${e}`,
    };
    return globalCache[key];
  }

  const dependencies: { [alias: string]: PackageKey } = {};
  const manifestDependencies = manifest.dependencies || {};
  for (const alias of Object.keys(manifestDependencies)) {
    const dependencyDefinition = manifestDependencies[alias];
    if ("path" in dependencyDefinition && "path" in thisPkg) {
      // Needs to be canonical
      dependencyDefinition.path = resolvePath(
        thisPkg.path,
        dependencyDefinition.path,
      )!;
    }
    dependencies[alias] = getKeyForDependencyDefinition(dependencyDefinition);
  }

  const packageInfo = {
    sources,
    languageFeatures: manifest.languageFeatures || [],
    dependencies,
  };
  log.debug(
    `resolved package info for dep ${key}: ${JSON.stringify(packageInfo)}`,
  );
  const pkg = {
    packageInfo,
    manifest,
  };

  log.trace(`adding package ${key} to cache`);
  globalCache[key] = pkg;
  return pkg;
}

function resolvePath(base: string, relative: string): string | null {
  try {
    return Utils.resolvePath(Uri.parse(base, true), relative).toString();
  } catch (e) {
    log.warn(`Failed to resolve path ${base} and ${relative}: ${e}`);
    return null;
  }
}

async function githubFetch(url: string) {
  const resp = await fetch(url, {
    method: "GET",
    headers: {
      Accept: "application/vnd.github.raw+json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });

  if (!resp.ok) {
    log.warn(`githubFetch: ${url} -> ${resp.status} ${resp.statusText}`);
  }

  resp.headers.forEach((value, name) => {
    log.info(`githubFetch response header: ${name}: ${value}`);
  });

  return resp;
}

async function readGithubManifestAndSources(github: {
  owner: string;
  repo: string;
  ref: string;
  path?: string;
}): Promise<
  { sources: [string, string][]; manifestContent: string } | { error: string }
> {
  // https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28#get-a-tree
  // curl -L \
  //   -H "Accept: application/vnd.github+json" \
  //   -H "Authorization: Bearer <YOUR-TOKEN>" \
  //   -H "X-GitHub-Api-Version: 2022-11-28" \
  //   https://api.github.com/repos/OWNER/REPO/git/trees/TREE_SHA
  const response = await fetch(
    `https://api.github.com/repos/${github.owner}/${github.repo}/git/trees/${github.ref}?recursive=1`,
    {
      method: "GET",
      headers: {
        Accept: "application/vnd.github.raw+json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
    },
  );
  log.info(`fetching github dependency ${github}`);
  if (!response.ok) {
    return {
      error: `Failed to fetch github dependency ${github}: ${response.status} ${response.statusText}`,
    };
  }

  const res = await response.json();
  log.info(`got tree: ${JSON.stringify(res)}`);
  const tree = res.tree as {
    path: string;
    mode: string;
    type: string;
    sha: string;
    url: string;
  }[];
  const subtree = tree.filter((entry) =>
    entry.path.startsWith(github.path ?? ""),
  );
  const qsharpJsonPath = (github.path ?? "") + "qsharp.json";
  const qsharpJsonEntry = subtree.find(
    (entry) => entry.path === qsharpJsonPath,
  );

  if (!qsharpJsonEntry) {
    return {
      // TODO: unacceptable! handle large trees (maybe by requiring sign in)
      error: res.truncated
        ? `${github} has too many files, try with a smaller repo`
        : `No qsharp.json found for github dependency ${github}`,
    };
  }

  const manifestContentsResponse = await fetch(qsharpJsonEntry.url, {
    method: "GET",
    headers: {
      Accept: "application/vnd.github.raw+json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });
  if (!manifestContentsResponse.ok) {
    return {
      error: `Failed to fetch qsharp.json for dependency ${github}: ${manifestContentsResponse.status} ${manifestContentsResponse.statusText}`,
    };
  }

  log.info(`found manifest at ${qsharpJsonEntry.url}`);

  const srcPrefix = (github.path ?? "") + "src/";
  const srcEntries = subtree.filter(
    (entry) =>
      entry.path.startsWith(srcPrefix) &&
      entry.type === "blob" &&
      entry.path.endsWith(".qs"),
  );

  log.info(`found sources ${JSON.stringify(srcEntries.map((s) => s.path))}`);

  let manifestContent: string;
  try {
    manifestContent = await manifestContentsResponse.text();
  } catch (e) {
    return {
      error: `Failed to read qsharp.json for dependency ${github}: ${e}`,
    };
  }

  const sources: [string, string][] = [];
  for (const srcEntry of srcEntries) {
    const srcContentsResponse = await githubFetch(srcEntry.url);
    if (!srcContentsResponse.ok) {
      return {
        error: `Failed to fetch source file ${srcEntry.path}: ${srcContentsResponse.status} ${srcContentsResponse.statusText}`,
      };
    }
    const srcContents = await srcContentsResponse.text();

    try {
      // TODO: this doesn't yet work because of implicit namespaces and the way I hackily concatenate the sourcemaps
      // const absolutePath = Uri.from({
      //   scheme: "qsharp-github-source",
      //   authority: key,
      //   path: "/" + srcEntry.path,
      // }).toString();
      sources.push(["GitHub" + "/" + srcEntry.path, srcContents]);
    } catch (e) {
      return {
        error: `failed to create uri from ${github} and /${srcEntry.path}: ${e}`,
      };
    }
  }

  return { sources, manifestContent };
}

async function readLocalManifestAndSources(
  directory: string,
): Promise<
  { sources: [string, string][]; manifestContent: string } | { error: string }
> {
  const manifestPath = resolvePath(directory, "qsharp.json");

  if (!manifestPath) {
    return {
      error: `Could not find qsharp.json in directory ${directory}`,
    };
  }

  const manifestContent = await readFile(manifestPath);
  if (!manifestContent) {
    return {
      error: `Could not read qsharp.json in directory ${directory}`,
    };
  }

  if (!projectLoader) {
    projectLoader = await getProjectLoader(
      readFile,
      listDir,
      async ([a, b]) => resolvePath(a, b) || "",
    );
  }
  const sources = await projectLoader.load_project({
    manifestDirectory: directory,
  });

  return {
    manifestContent,
    sources,
  };
}

function getKeyForDependencyDefinition(
  dependencyDefinition: DependencyDefinition,
): PackageKey {
  const key = JSON.stringify(dependencyDefinition);
  return key;
}

function decodeDependencyDefinitionFromKey(key: string): DependencyDefinition {
  log.trace(`decodeDependencyDefinitionFromKey: ${key} -> { path: ${key} }`);
  return JSON.parse(key);
}
