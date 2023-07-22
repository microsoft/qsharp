// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

// See docs at https://code.visualstudio.com/api/extension-guides/tree-view

let resourcesPath: vscode.Uri;

export class WorkspaceTreeProvider
  implements vscode.TreeDataProvider<Workspace>
{
  private _onDidChangeData = new vscode.EventEmitter<Workspace | undefined>();
  readonly onDidChangeTreeData: vscode.Event<Workspace | undefined> =
    this._onDidChangeData.event;

  constructor(context: vscode.ExtensionContext) {
    resourcesPath = vscode.Uri.joinPath(context.extensionUri, "resources");
  }
  async refresh() {
    this._onDidChangeData.fire(undefined);
  }

  getTreeItem(element: Workspace): vscode.TreeItem | Thenable<vscode.TreeItem> {
    return element;
  }
  getChildren(
    element?: Workspace | undefined
  ): vscode.ProviderResult<Workspace[]> {
    if (!element) {
      return [
        new Workspace("workspace", "Chemistry", true, "workspace.svg"),
        new Workspace("workspace", "Research", true, "workspace.svg"),
      ];
    } else if (element.label === "Chemistry") {
      return [
        new Workspace("header", "Targets", true),
        new Workspace("header", "Jobs", true),
        new Workspace("header", "Results", true),
      ];
    } else if (element.label === "Targets") {
      return [
        new Workspace("target", "IonQ", false, "atom.svg"),
        new Workspace("target", "Quantinuum", false, "atom.svg"),
        new Workspace("target", "Rigetti", false, "atom.svg"),
      ];
    } else if (element.label === "Jobs") {
      return [new Workspace("job", "hydrogen-2", false, "job.svg")];
    } else if (element.label === "Results") {
      return [
        new Workspace("result", "analysis-1", false, "check.svg"),
        new Workspace("result", "qrng-estimate", false, "check.svg"),
      ];
    } else {
      return [];
    }
  }
}

class Workspace extends vscode.TreeItem {
  constructor(type: string, label: string, expand: boolean, icon?: string) {
    super(
      label,
      expand
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None
    );
    this.contextValue = type;
    if (icon) {
      this.iconPath = {
        light: vscode.Uri.joinPath(resourcesPath, "light", icon),
        dark: vscode.Uri.joinPath(resourcesPath, "dark", icon),
      };
    }
    if (this.label === "IonQ") {
      const status = new vscode.MarkdownString(`
__Status__: Online<br>
__Queue time__: 2 hours
      `);
      status.supportHtml = true;
      this.tooltip = status;
    }
    if (this.label === "Chemistry") {
      const hover = new vscode.MarkdownString(`
__Quota remaining__: $500.00
  `);
      hover.supportHtml = true;
      this.tooltip = hover;
    }
    if (type === "job") {
      const hover = new vscode.MarkdownString(
        `__Submitted__: 2023-06-25, 15:34 UTC`
      );
      hover.supportHtml = true;
      this.tooltip = hover;
    }
    if (type === "result") {
      const hover = new vscode.MarkdownString(
        `__Submitted__: 2023-06-25, 15:34 UTC<br>
__Completed__: 2023-06-25, 15:45 UTC<br>
__Result__: Success<br>
__Size__: 10kb
        `
      );
      hover.supportHtml = true;
      this.tooltip = hover;
    }
  }
}
