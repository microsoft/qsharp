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
  const docUri = vscode.Uri.joinPath(workspaceFolderUri, "test.qs");
  const projectMainDocUri = vscode.Uri.joinPath(
    workspaceFolderUri,
    "packages",
    // TODO: I wanted to call this main-package but my current hacky way of
    // concatenating source lists makes that an invalid namespace way.
    // Change it back when we have things properly implemented so we can validate
    // that we support dashes in project folder names.
    "MainPackage",
    "src",
    "Main.qs",
  );
  const projectDepDocUri = vscode.Uri.joinPath(
    workspaceFolderUri,
    "packages",
    // TODO: I wanted to call this main-package but my current hacky way of
    // concatenating source lists makes that an invalid namespace way.
    // Change it back when we have things properly implemented so we can validate
    // that we support dashes in project folder names.
    "DepPackage",
    "src",
    "Main.qs",
  );

  this.beforeAll(async () => {
    await activateExtension();

    // Pre-open the text documents that are going to be interacted with in
    // the tests. This just gives the language a service a bit of time to load
    // fully in the background before the test cases run. Is it a hack? Absolutely.
    // But we don't currently have to await background language service tasks.
    // This is the best we can do to ensure the features have been initialized
    // before we start testing.
    await vscode.workspace.openTextDocument(docUri);
    await vscode.workspace.openTextDocument(projectMainDocUri);
    // Give it a tiny bit of time to settle
    await new Promise((resolve) => setTimeout(resolve, 50));
  });

  test("Q# language is registered", async () => {
    const doc = await vscode.workspace.openTextDocument(docUri);
    assert.equal(
      doc.languageId,
      "qsharp",
      "document language should be `qsharp`",
    );
  });

  test("Completions", async () => {
    const actualCompletionList = (await vscode.commands.executeCommand(
      "vscode.executeCompletionItemProvider",
      docUri,
      new vscode.Position(0, 0),
    )) as vscode.CompletionList;

    assert.include(
      actualCompletionList.items.map((i) => i.label),
      "operation",
    );
  });

  test("Definition", async () => {
    const doc = await vscode.workspace.openTextDocument(docUri);
    const text = doc.getText(
      new vscode.Range(new vscode.Position(4, 16), new vscode.Position(4, 19)),
    );
    // Sanity check the test setup - is this the correct position?
    assert.equal(text, "foo");

    const actualDefinition = (await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      docUri,
      new vscode.Position(4, 18), // cursor on the usage of foo
    )) as vscode.Location[];

    const location = actualDefinition[0];
    assert.equal(location.uri.toString(), docUri.toString());
    assert.equal(location.range.start.line, 3);
    assert.equal(location.range.start.character, 12);
  });

  test("Diagnostics", async () => {
    const actualDiagnostics = vscode.languages.getDiagnostics(docUri);
    assert.lengthOf(actualDiagnostics, 1);

    assert.include(actualDiagnostics[0].message, "syntax error");
    assert.equal(actualDiagnostics[0].range.start.line, 7);
  });

  test("Hover", async () => {
    const doc = await vscode.workspace.openTextDocument(docUri);
    const text = doc.getText(
      new vscode.Range(new vscode.Position(4, 16), new vscode.Position(4, 19)),
    );
    // Sanity check the test setup - is this the correct position?
    assert.equal(text, "foo");

    const actualHovers = (await vscode.commands.executeCommand(
      "vscode.executeHoverProvider",
      docUri,
      new vscode.Position(4, 18), // cursor on the usage of foo
    )) as vscode.Hover[];

    assert.lengthOf(actualHovers, 1);
    assert.lengthOf(actualHovers[0].contents, 1);
    const md = actualHovers[0].contents[0] as vscode.MarkdownString;
    assert.include(md.value, "foo : String");
  });

  test("Signature Help", async () => {
    const doc = await vscode.workspace.openTextDocument(docUri);
    const text = doc.getText(
      new vscode.Range(new vscode.Position(4, 16), new vscode.Position(4, 19)),
    );
    // Sanity check the test setup - is this the correct position?
    assert.equal(text, "foo");

    const actualSignatureHelp = (await vscode.commands.executeCommand(
      "vscode.executeSignatureHelpProvider",
      docUri,
      new vscode.Position(4, 18), // cursor on the usage of foo
    )) as vscode.SignatureHelp;

    assert.lengthOf(actualSignatureHelp.signatures, 1);
    assert.include(
      actualSignatureHelp.signatures[0].label,
      "function Message(msg : String)",
    );
  });

  test("Format Document", async () => {
    await vscode.workspace.openTextDocument(docUri);

    const actualFormatEdits = (await vscode.commands.executeCommand(
      "vscode.executeFormatDocumentProvider",
      docUri,
    )) as vscode.TextEdit[];

    assert.lengthOf(actualFormatEdits, 1);
    assert.equal(actualFormatEdits[0].range.start.line, 7);
    assert.equal(actualFormatEdits[0].range.start.character, 27);
    assert.equal(actualFormatEdits[0].range.end.line, 8);
    assert.equal(actualFormatEdits[0].range.end.character, 4);
    assert.equal(actualFormatEdits[0].newText, "");
  });

  test("Format Document Range", async () => {
    await vscode.workspace.openTextDocument(docUri);

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
      docUri,
      noEditRange,
    )) as vscode.TextEdit[];

    // assert that edits outside the range aren't returned
    assert.isUndefined(actualFormatEdits);

    actualFormatEdits = (await vscode.commands.executeCommand(
      "vscode.executeFormatRangeProvider",
      docUri,
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
    const doc = await vscode.workspace.openTextDocument(docUri);

    const actualCodeLenses = (await vscode.commands.executeCommand(
      "vscode.executeCodeLensProvider",
      docUri,
    )) as vscode.CodeLens[];

    assert.lengthOf(actualCodeLenses, 5);

    for (const lens of actualCodeLenses) {
      assert.include(doc.getText(lens.range), "operation Test()");
    }
  });

  test("Package dependencies", async () => {
    const doc = await vscode.workspace.openTextDocument(projectMainDocUri);
    vscode.commands.executeCommand("workbench.action.problems.focus");

    // Sanity check the test setup - is this the correct position?
    const text = doc.getText(
      new vscode.Range(new vscode.Position(1, 10), new vscode.Position(1, 20)),
    );
    assert.equal(text, "MyFunction");

    // Verify go-to-definition works across packages
    const actualDefinition = (await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      projectMainDocUri,
      new vscode.Position(1, 15), // cursor on the usage of "MyFunction"
    )) as vscode.Location[];

    // Returned location should be in DepPackage on teh definition of "MyFunction"
    assert.lengthOf(actualDefinition, 1);
    const location = actualDefinition[0];
    assert.equal(location.uri.toString(), projectDepDocUri.toString());
    assert.equal(location.range.start.line, 1);
    assert.equal(location.range.start.character, 13);

    // No errors if package dependencies are properly resolved
    const actualDiagnostics =
      vscode.languages.getDiagnostics(projectMainDocUri);
    assert.isEmpty(actualDiagnostics);
  });
});
