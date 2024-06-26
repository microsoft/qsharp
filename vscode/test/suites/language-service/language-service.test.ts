// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";
import { activateExtension } from "../extensionUtils";

suite("Q# Language Service Tests", function suite() {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");

  const workspaceFolderUri = workspaceFolder.uri;
  const testQs = vscode.Uri.joinPath(workspaceFolderUri, "test.qs");
  const noErrorsQs = vscode.Uri.joinPath(workspaceFolderUri, "no-errors.qs");

  this.beforeAll(async () => {
    await activateExtension();
  });

  test("Q# language is registered", async () => {
    const doc = await vscode.workspace.openTextDocument(testQs);
    assert.equal(
      doc.languageId,
      "qsharp",
      "document language should be `qsharp`",
    );
  });

  test("Completions", async () => {
    const actualCompletionList = (await vscode.commands.executeCommand(
      "vscode.executeCompletionItemProvider",
      testQs,
      new vscode.Position(0, 0),
    )) as vscode.CompletionList;

    assert.include(
      actualCompletionList.items.map((i) => i.label),
      "operation",
    );
  });

  test("Definition", async () => {
    const doc = await vscode.workspace.openTextDocument(testQs);
    const text = doc.getText(
      new vscode.Range(new vscode.Position(4, 16), new vscode.Position(4, 19)),
    );
    // Sanity check the test setup - is this the correct position?
    assert.equal(text, "foo");

    const actualDefinition = (await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      testQs,
      new vscode.Position(4, 18), // cursor on the usage of foo
    )) as vscode.Location[];

    const location = actualDefinition[0];
    assert.equal(location.uri.toString(), testQs.toString());
    assert.equal(location.range.start.line, 3);
    assert.equal(location.range.start.character, 12);
  });

  test("Diagnostics", async () => {
    const actualDiagnostics = vscode.languages.getDiagnostics(testQs);
    assert.lengthOf(actualDiagnostics, 1);

    assert.include(actualDiagnostics[0].message, "syntax error");
    assert.equal(actualDiagnostics[0].range.start.line, 7);
  });

  test("Hover", async () => {
    const doc = await vscode.workspace.openTextDocument(testQs);
    const text = doc.getText(
      new vscode.Range(new vscode.Position(4, 16), new vscode.Position(4, 19)),
    );
    // Sanity check the test setup - is this the correct position?
    assert.equal(text, "foo");

    const actualHovers = (await vscode.commands.executeCommand(
      "vscode.executeHoverProvider",
      testQs,
      new vscode.Position(4, 18), // cursor on the usage of foo
    )) as vscode.Hover[];

    assert.lengthOf(actualHovers, 1);
    assert.lengthOf(actualHovers[0].contents, 1);
    const md = actualHovers[0].contents[0] as vscode.MarkdownString;
    assert.include(md.value, "foo : String");
  });

  test("Signature Help", async () => {
    const doc = await vscode.workspace.openTextDocument(testQs);
    const text = doc.getText(
      new vscode.Range(new vscode.Position(4, 16), new vscode.Position(4, 19)),
    );
    // Sanity check the test setup - is this the correct position?
    assert.equal(text, "foo");

    const actualSignatureHelp = (await vscode.commands.executeCommand(
      "vscode.executeSignatureHelpProvider",
      testQs,
      new vscode.Position(4, 18), // cursor on the usage of foo
    )) as vscode.SignatureHelp;

    assert.lengthOf(actualSignatureHelp.signatures, 1);
    assert.include(
      actualSignatureHelp.signatures[0].label,
      "function Message(msg : String)",
    );
  });

  test("Format Document", async () => {
    await vscode.workspace.openTextDocument(testQs);

    const actualFormatEdits = (await vscode.commands.executeCommand(
      "vscode.executeFormatDocumentProvider",
      testQs,
    )) as vscode.TextEdit[];

    assert.lengthOf(actualFormatEdits, 1);
    assert.equal(actualFormatEdits[0].range.start.line, 7);
    assert.equal(actualFormatEdits[0].range.start.character, 27);
    assert.equal(actualFormatEdits[0].range.end.line, 8);
    assert.equal(actualFormatEdits[0].range.end.character, 4);
    assert.equal(actualFormatEdits[0].newText, "");
  });

  test("Format Document Range", async () => {
    await vscode.workspace.openTextDocument(testQs);

    const noEditRange = new vscode.Range(
      new vscode.Position(7, 24),
      new vscode.Position(7, 27),
    );
    const editRange = new vscode.Range(
      new vscode.Position(7, 27),
      new vscode.Position(8, 4),
    );

    let actualFormatEdits = (await vscode.commands.executeCommand(
      "vscode.executeFormatRangeProvider",
      testQs,
      noEditRange,
    )) as vscode.TextEdit[];

    // assert that edits outside the range aren't returned
    assert.isUndefined(actualFormatEdits);

    actualFormatEdits = (await vscode.commands.executeCommand(
      "vscode.executeFormatRangeProvider",
      testQs,
      editRange,
    )) as vscode.TextEdit[];

    assert.lengthOf(actualFormatEdits, 1);
    assert.equal(actualFormatEdits[0].range.start.line, 7);
    assert.equal(actualFormatEdits[0].range.start.character, 27);
    assert.equal(actualFormatEdits[0].range.end.line, 8);
    assert.equal(actualFormatEdits[0].range.end.character, 4);
    assert.equal(actualFormatEdits[0].newText, "");
  });

  test("Code Lens", async () => {
    const doc = await vscode.workspace.openTextDocument(noErrorsQs);

    const actualCodeLenses = (await vscode.commands.executeCommand(
      "vscode.executeCodeLensProvider",
      doc.uri,
    )) as vscode.CodeLens[];

    assert.lengthOf(actualCodeLenses, 5);

    for (const lens of actualCodeLenses) {
      assert.include(doc.getText(lens.range), "function Test()");
    }
  });
});
