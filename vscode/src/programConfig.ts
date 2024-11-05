// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IProjectConfig, ProgramConfig } from "qsharp-lang";
import * as vscode from "vscode";
import { isQsharpDocument } from "./common";
import { getTarget } from "./config";
import { loadProject } from "./projectSystem";

/**
 * Notice the similarity to @type {ProgramConfig} and @type {IProjectConfig}.
 * These types look similar but have a few differences:
 *
 * ProgramConfig is used in the API for the qsharp-lang package. All properties
 * are optional for backward compatibility. It only contains the properties
 * that are needed by the compiler APIs.
 *
 * ProjectConfig contains the values come from the Q# manifest, or for single-file
 * programs, the defaults for these values. In the future, for projects with
 * dependencies, it will also contain the full dependency graph and sources for
 * all packages referenced by the project.
 *
 * FullProgramConfig is a union of the above. It's meant to represent a fully
 * populated configuration that can be used across a variety of extension features.
 * So all the properties are required.
 */

export type FullProgramConfig = Required<ProgramConfig & IProjectConfig>;

type FullProgramConfigOrError =
  | {
      success: true;
      programConfig: FullProgramConfig;
    }
  | {
      success: false;
      errorMsg: string;
    };

/**
 * @returns The currently active Q# project configuration. This is a general-purpose
 *   function that is useful for any extension command that is intended to
 *   operate on the "current" project.
 */
export async function getActiveProgram(): Promise<FullProgramConfigOrError> {
  const docUri = getActiveQSharpDocumentUri();
  if (!docUri) {
    return {
      success: false,
      errorMsg: "The currently active window is not a Q# document",
    };
  }

  return await getProgramForDocument(docUri);
}

/**
 * @returns The URI of the currently active Q# document, or undefined if the current
 *   active editor is not a Q# document.
 */
export function getActiveQSharpDocumentUri(): vscode.Uri | undefined {
  const editor = vscode.window.activeTextEditor;

  return editor?.document && isQsharpDocument(editor.document)
    ? editor.document.uri
    : undefined;
}

/**
 * @param docUri A Q# document URI.
 * @returns The program configuration that applies to this document,
 *   including any settings that come from the qsharp.json as well as the
 *   user/workspace settings.
 */
export async function getProgramForDocument(
  docUri: vscode.Uri,
): Promise<FullProgramConfigOrError> {
  // Target profile comes from settings
  const profile = getTarget();

  // Project configs come from the document and/or manifest
  try {
    const program = await loadProject(docUri);

    return { success: true, programConfig: { profile, ...program } };
  } catch (e: any) {
    return {
      success: false,
      errorMsg: e.message || "Failed to load Q# project",
    };
  }
}
