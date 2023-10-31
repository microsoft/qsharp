// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { assert } from "chai";

suite("Q# Language Service Tests", () => {
  const workspaceFolder =
    vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
  assert(workspaceFolder, "Expecting an open folder");

  const workspaceFolderUri = workspaceFolder.uri;
  const docUri = vscode.Uri.joinPath(workspaceFolderUri, "test.qs");

  test("Q# language is registered", async () => {
    const doc = await vscode.workspace.openTextDocument(docUri);
    assert.equal(
      doc.languageId,
      "qsharp",
      "document language should be `qsharp`",
    );
  });

  test("Completions", async () => {
    await activate();
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
    await activate();
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
    await activate();

    const actualDiagnostics = vscode.languages.getDiagnostics(docUri);
    assert.lengthOf(actualDiagnostics, 1);

    assert.include(actualDiagnostics[0].message, "syntax error");
    assert.equal(actualDiagnostics[0].range.start.line, 7);
  });

  test("Hover", async () => {
    await activate();
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
    await activate();
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
});

async function activate() {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const ext = vscode.extensions.getExtension("quantum.qsharp-lang-vscode-dev")!;
  await ext.activate();
}
