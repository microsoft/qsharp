// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IProjectConfig, ProgramConfig } from "qsharp-lang";
import * as vscode from "vscode";
import { isOpenQasmDocument, isQdkDocument } from "./common";
import { getTarget } from "./config";
import { loadOpenQasmProject, loadProject } from "./projectSystem";

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

export type FullProgramConfigOrError =
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
  const doc = getActiveQdkDocument();
  if (!doc) {
    return {
      success: false,
      errorMsg:
        "The currently active window is not a document supported by the QDK",
    };
  }

  return await getProgramForDocument(doc);
}

export function getActiveQdkDocumentUri(): vscode.Uri | undefined {
  return getActiveQdkDocument()?.uri;
}

export function getActiveQdkDocument(): vscode.TextDocument | undefined {
  const editor = vscode.window.activeTextEditor;
  return editor?.document && isQdkDocument(editor.document)
    ? editor.document
    : undefined;
}

export async function getVisibleProgram(): Promise<FullProgramConfigOrError> {
  const doc = getVisibleQdkDocument();
  if (!doc) {
    return {
      success: false,
      errorMsg:
        "There are no visible windows that contain a document supported by the QDK",
    };
  }
  return await getProgramForDocument(doc);
}

export function getVisibleQdkDocumentUri(): vscode.Uri | undefined {
  return getVisibleQdkDocument()?.uri;
}

function getVisibleQdkDocument(): vscode.TextDocument | undefined {
  return vscode.window.visibleTextEditors.find((editor) =>
    isQdkDocument(editor.document),
  )?.document;
}

export async function getProgramForDocument(doc: vscode.TextDocument) {
  return isOpenQasmDocument(doc)
    ? await getOpenQasmProgramForDocument(doc.uri)
    : await getQSharpProgramForDocument(doc.uri);
}

/**
 * @param docUri A Q# document URI.
 * @returns The program configuration that applies to this document,
 *   including any settings that come from the qsharp.json as well as the
 *   user/workspace settings.
 */
async function getQSharpProgramForDocument(
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

/**
 * @param docUri An OpenQASM document URI.
 * @returns The program configuration that applies to this document,
 *   with user/workspace settings.
 */
async function getOpenQasmProgramForDocument(
  docUri: vscode.Uri,
): Promise<FullProgramConfigOrError> {
  // Target profile comes from settings
  const profile = getTarget();

  // Project configs come from the document
  try {
    const program = await loadOpenQasmProject(docUri);

    return { success: true, programConfig: { profile, ...program } };
  } catch (e: any) {
    return {
      success: false,
      errorMsg: e.message || "Failed to load OpenQASM project",
    };
  }
}
