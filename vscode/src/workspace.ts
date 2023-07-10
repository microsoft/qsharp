// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp";

import {
  AzureSubscription,
  VSCodeAzureSubscriptionProvider,
} from "@microsoft/vscode-azext-azureauth";
import { WorkspaceTreeProvider } from "./workspaceTree";

const mgmtEndpoint = "https://management.azure.com";

type workspacesResponse = {
  value: Array<{
    id: string;
    name: string;
    location: string;
    properties: {
      providers: Array<{
        providerId: string; // "ionq", "quantinumm", etc.
      }>;
    };
  }>;
};

let workspaceTreeProvider: WorkspaceTreeProvider;

async function useNewSdk(): Promise<AzureSubscription[]> {
  const xx = new VSCodeAzureSubscriptionProvider();
  const result = await xx.signIn();
  if (!result) {
    log.error("Unable to sign-in");
    return [];
  }
  const subscriptions = await xx.getSubscriptions();
  log.info(`Got ${subscriptions.length} subscriptions`);
  subscriptions.forEach((sub) => {
    log.info(`  name: "${sub.name}", id: "${sub.subscriptionId}"`);
  });
  return subscriptions;
}

export function setupWorkspaces(context: vscode.ExtensionContext) {
  workspaceTreeProvider = new WorkspaceTreeProvider(context);
  const workspaceTree = vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });

  workspaceTree.onDidChangeSelection((evt) => {
    if (evt.selection.length) {
      log.debug("TreeView selection changed to ", evt.selection[0].label);
      evt.selection[0];
    }
  });

  vscode.commands.registerCommand("quantum-workspaces-refresh", () => {
    workspaceTreeProvider.refresh();
  });

  vscode.commands.registerCommand(
    "extension.qsharp.listWorkspaces",
    async () => {
      const subs = await useNewSdk();
      if (subs.length) {
        const sub = subs[0];
        // TODO: Prompt for which subscription to use
        const session = await sub.authentication.getSession([
          "https://management.azure.com/.default",
        ]);
        const accessToken = session?.accessToken;
        if (!accessToken) {
          log.error("No access token in the session");
          return;
        }

        // TODO: Should really use one of the Azure SDKs for making requests.
        const path = `/subscriptions/${sub.subscriptionId}/providers/Microsoft.Quantum/workspaces`;
        const restUri = `${mgmtEndpoint}${path}?api-version=2022-01-10-preview`;
        const restResponse = await fetch(restUri, {
          headers: [
            ["Authorization", `Bearer ${accessToken}`],
            ["Content-Type", "application/json"],
          ],
          method: "GET",
        });
        if (restResponse.ok) {
          const json: workspacesResponse = await restResponse.json();
          log.debug("Subscriptions response: ", json);
        } else {
          const body = await restResponse.text();
          log.error(
            "Subscriptions request failed with: ",
            restResponse.status,
            restResponse.statusText,
            body
          );
        }
      }
    }
  );
}
