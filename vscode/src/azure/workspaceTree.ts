// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

// See docs at https://code.visualstudio.com/api/extension-guides/tree-view

// Convert a date such as "2023-07-24T17:25:09.1309979Z" into local time
function localDate(date: string) {
  return new Date(date).toLocaleString();
}

export class WorkspaceTreeProvider
  implements vscode.TreeDataProvider<WorkspaceTreeItem>
{
  private treeState: Map<string, WorkspaceConnection> = new Map();

  private _onDidChangeData = new vscode.EventEmitter<
    WorkspaceTreeItem | undefined
  >();
  readonly onDidChangeTreeData: vscode.Event<WorkspaceTreeItem | undefined> =
    this._onDidChangeData.event;

  async updateWorkspace(workspace: WorkspaceConnection) {
    this.treeState.set(workspace.id, workspace);
    this.refresh();
  }

  async refresh() {
    this._onDidChangeData.fire(undefined);
  }

  getTreeItem(
    element: WorkspaceTreeItem
  ): vscode.TreeItem | Thenable<vscode.TreeItem> {
    return element;
  }
  getChildren(
    element?: WorkspaceTreeItem | undefined
  ): vscode.ProviderResult<WorkspaceTreeItem[]> {
    if (!element) {
      return [...this.treeState.values()].map(
        (workspace) =>
          new WorkspaceTreeItem(
            workspace.name,
            workspace,
            "workspace",
            workspace
          )
      );
    } else {
      return element.getChildren();
    }
  }
}

type Target = {
  providerId: string;
  provisioningState: string;
  status?: "Online" | "Offline";
  queueTime?: number;
};

export type Job = {
  id: string;
  name: string;
  target: string;
  status: "Waiting" | "Executing" | "Succeeded" | "Failed" | "Cancelled";
  outputDataUri?: string;
  creationTime: string;
  beginExecutionTime?: string;
  endExecutionTime?: string;
  cancellationTime?: string;
  costEstimate?: any;
};

export type WorkspaceConnection = {
  connection: any;
  id: string;
  name: string;
  storageAccount: string;
  endpointUri: string;
  tenantId: string;
  quota?: any;
  targets: Target[];
  jobs: Job[];
};

export class WorkspaceTreeItem extends vscode.TreeItem {
  constructor(
    label: string,
    public workspace: WorkspaceConnection,
    public type: "workspace" | "targetHeader" | "target" | "jobHeader" | "job",
    public itemData: WorkspaceConnection | Target[] | Target | Job[] | Job
  ) {
    super(label, vscode.TreeItemCollapsibleState.Collapsed);

    this.contextValue = type;

    switch (type) {
      case "workspace":
        this.iconPath = new vscode.ThemeIcon("notebook");
        break;
      case "target": {
        const target = itemData as Target;
        this.iconPath = new vscode.ThemeIcon("package");
        this.collapsibleState = vscode.TreeItemCollapsibleState.None;
        if (target.status || target.queueTime) {
          const hover = new vscode.MarkdownString(
            `${target.status ? `__Status__: ${target.status}<br>` : ""}
            ${
              target.queueTime
                ? `__Queue time__: ${target.queueTime}mins<br>`
                : ""
            }`
          );
          hover.supportHtml = true;
          this.tooltip = hover;
        }
        break;
      }
      case "job": {
        const job = itemData as Job;
        this.collapsibleState = vscode.TreeItemCollapsibleState.None;
        switch (job.status) {
          case "Executing":
            // this.iconPath = new vscode.ThemeIcon("debug-line-by-line");
            this.iconPath = new vscode.ThemeIcon("run-all");
            break;
          case "Waiting":
            // this.iconPath = new vscode.ThemeIcon("watch");
            this.iconPath = new vscode.ThemeIcon("loading~spin");
            break;
          case "Cancelled":
            this.iconPath = new vscode.ThemeIcon("circle-slash");
            this.contextValue = "result";
            break;
          case "Failed":
            this.iconPath = new vscode.ThemeIcon("error");
            this.contextValue = "result";
            break;
          case "Succeeded":
            this.iconPath = new vscode.ThemeIcon("pass");
            this.contextValue = "result";
            break;
        }
        // Tooltip
        const hover = new vscode.MarkdownString(
          `__Created__: ${localDate(job.creationTime)}<br>
          __Target__: ${job.target}<br>
          __Status__: ${job.status}<br>
          ${
            job.beginExecutionTime
              ? `__Started__: ${localDate(job.beginExecutionTime)}<br>`
              : ""
          }
          ${
            job.endExecutionTime
              ? `__Completed__: ${localDate(job.endExecutionTime)}<br>`
              : ""
          }
          ${
            job.costEstimate ? `__Cost estimate__: ${job.costEstimate}<br>` : ""
          }
        `
        );
        hover.supportHtml = true;
        this.tooltip = hover;
        break;
      }

      default:
        break;
    }
  }

  getChildren(): WorkspaceTreeItem[] {
    switch (this.type) {
      case "workspace":
        return [
          new WorkspaceTreeItem(
            "Targets",
            this.workspace,
            "targetHeader",
            this.workspace.targets
          ),
          new WorkspaceTreeItem(
            "Jobs",
            this.workspace,
            "jobHeader",
            this.workspace.jobs
          ),
        ];

      case "targetHeader":
        return (this.itemData as Target[]).map(
          (target) =>
            new WorkspaceTreeItem(
              target.providerId,
              this.workspace,
              "target",
              target
            )
        );
      case "jobHeader":
        return (this.itemData as Job[]).map(
          (job) =>
            new WorkspaceTreeItem(
              job.name || job.id,
              this.workspace,
              "job",
              job
            )
        );
      case "target":
      case "job":
      default:
        return [];
    }
  }
}

//     if (this.label === "IonQ") {
//       const status = new vscode.MarkdownString(`
// __Status__: Online<br>
// __Queue time__: 2 hours
//       `);
//       status.supportHtml = true;
//       this.tooltip = status;
//     }
//     if (this.label === "Chemistry") {
//       const hover = new vscode.MarkdownString(`
// __Quota remaining__: $500.00
//   `);
//       hover.supportHtml = true;
//       this.tooltip = hover;
//     }
//     if (type === "job") {
//       const hover = new vscode.MarkdownString(
//         `__Submitted__: 2023-06-25, 15:34 UTC`
//       );
//       hover.supportHtml = true;
//       this.tooltip = hover;
//     }
//     if (type === "result") {
//       const hover = new vscode.MarkdownString(
//         `__Submitted__: 2023-06-25, 15:34 UTC<br>
// __Completed__: 2023-06-25, 15:45 UTC<br>
// __Result__: Success<br>
// __Size__: 10kb
//         `
//       );
//       hover.supportHtml = true;
//       this.tooltip = hover;
//     }
