import * as vscode from "vscode";
import { log } from "qsharp";

import {
  AzureSubscription,
  VSCodeAzureSubscriptionProvider,
} from "@microsoft/vscode-azext-azureauth";

const quantumScopes = [
  // "https://management.azure.com/.default",
  // "https://management.azure.com/user_impersonation",
  // "https://quantum.microsoft.comJobs.ReadWrite",
  "VSCODE_CLIENT_ID:cd2b63f7-1573-4507-b9ad-8329e3a38a91",
  "VSCODE_TENANT:consumers",
  "api://cd2b63f7-1573-4507-b9ad-8329e3a38a91/.default",
];
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

/****** A graveyard of failed experiments ******/

//   log.info("About to request an auth session for quantum scopes");
//   const authSession = await vscode.authentication.getSession(
//     "microsoft",
//     quantumScopes,
//     { createIfNone: true }
//     //{ createIfNone: false, silent: true }
//   );
//   log.trace("AuthSession returned: ", authSession);

//if (await useNewSdk()) return;
//return;

//   if (authSession?.accessToken) {
//     const token = authSession.accessToken;

//     // Fetch the subscriptions: https://learn.microsoft.com/en-us/rest/api/resources/subscriptions/list?tabs=HTTP
//     const uri = `${mgmtEndpoint}/subscriptions?api-version=2020-01-01`;
//     const response = await fetch(uri, {
//       headers: [
//         ["Authorization", `Bearer ${token}`],
//         ["Content-Type", "application/json"],
//       ],
//       method: "GET",
//     });
//     if (response.ok) {
//       const json = await response.json();
//       log.info("Subscriptions response: ", json);
//     } else {
//       const body = await response.text();
//       log.error(
//         "Subscriptions request failed with: ",
//         response.status,
//         response.statusText,
//         body
//       );
//     }

// Get the workspaces for the subscription
// See example response at https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/resource-manager/Microsoft.Quantum/preview/2022-01-10-preview/examples/quantumWorkspacesListSubscription.json
//         const myTestSub = `02e0a16f-334e-47a5-8672-d94e1ebee1b1`;
//         const path = `/subscriptions/${myTestSub}/providers/Microsoft.Quantum/workspaces`;
//         const workspacesUri = `${mgmtEndpoint}${path}?api-version=2022-01-10-preview`;
//         const workspaceResponse = await fetch(workspacesUri, {
//           headers: [
//             ["Authorization", `Bearer ${token}`],
//             ["Content-Type", "application/json"],
//           ],
//           method: "GET",
//         });
//         if (workspaceResponse.ok) {
//           const json = await workspaceResponse.json();
//           log.info("Subscriptions response: ", json);
//         } else {
//           const body = await workspaceResponse.text();
//           log.error(
//             "Subscriptions request failed with: ",
//             workspaceResponse.status,
//             workspaceResponse.statusText,
//             body
//           );
//         }
//       }
//     }
//   );
// }
