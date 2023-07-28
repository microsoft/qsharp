// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp";
import { AzureUris, scopes, ResponseTypes } from "./azure";
import * as vscode from "vscode";

export async function queryWorkspaces() {
  // *** Authenticate and retrieve tenants the user has Azure resources for ***

  // For the MSA case, you need to query the tenants first and get the underlying AzureAD
  // tenant for the 'guest' MSA. See https://stackoverflow.microsoft.com/a/76246/108570
  const firstAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.armMgmt],
    { createIfNone: true }
  );

  let response = await fetch(AzureUris.tenants(), {
    headers: [
      ["Authorization", `Bearer ${firstAuth.accessToken}`],
      ["Content-Type", "application/json"],
    ],
    method: "GET",
  });
  if (!response.ok) throw "Failed to get tenants";

  const tenantsObj = (await response.json()) as ResponseTypes.TenantList;
  if (!tenantsObj?.value?.length) throw "No tenants returned";

  // TODO: Quick-pick if more than one
  const tenantId = tenantsObj.value[0].tenantId;

  // *** Sign-in to that tenant and query the subscriptions available for it ***

  const tenantAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.armMgmt, `VSCODE_TENANT:${tenantId}`],
    { createIfNone: true }
  );

  response = await fetch(AzureUris.subscriptions(), {
    headers: [
      ["Authorization", `Bearer ${tenantAuth.accessToken}`],
      ["Content-Type", "application/json"],
    ],
    method: "GET",
  });
  if (!response.ok) throw "Failed to get subscriptions";

  const subsObj = (await response.json()) as ResponseTypes.SubscriptionList;
  if (!subsObj?.value?.length) throw "No subscriptions returned";

  // TODO: Quick-pick if more than one
  const subId = subsObj.value[0].subscriptionId;

  // *** Fetch the Quantum Workspaces in the subscription ***

  response = await fetch(AzureUris.workspaces(subId), {
    headers: [
      ["Authorization", `Bearer ${tenantAuth.accessToken}`],
      ["Content-Type", "application/json"],
    ],
    method: "GET",
  });
  if (!response.ok) throw "Failed to get workspaces";
  const workspacesObj = (await response.json()) as ResponseTypes.WorkspaceList;
  if (!workspacesObj.value.length) throw "Failed to get any workspaces";

  // id will be similar to: "/subscriptions/00000000-1111-2222-3333-444444444444/resourceGroups/quantumResourcegroup/providers/Microsoft.Quantum/Workspaces/quantumworkspace1"
  // endpointUri will be like: "https://quantumworkspace1.westus.quantum.azure.com" (but first segment should be removed)

  log.info(`Workspaces: ${JSON.stringify(workspacesObj, null, 2)}`);
  const workspace = workspacesObj.value[0];

  // Need to remove the first part of the endpoint
  const fixedEndpoint = workspace.properties.endpointUri.replace(
    `https://${workspace.name}.`,
    "https://"
  );

  // *** Query the workspace for its properties ***

  queryWorkspace(fixedEndpoint, workspace.id, tenantId);
}

// Reference for existing queries in Python SDK and Azure schema:
// - https://github.com/microsoft/qdk-python/blob/main/azure-quantum/azure/quantum/_client/aio/operations/_operations.py
// - https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/data-plane/Microsoft.Quantum/preview/2022-09-12-preview/quantum.json
export async function queryWorkspace(
  endpointUri: string,
  workspaceUri: string,
  tenantId: string
) {
  const workspaceAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.quantum, `VSCODE_TENANT:${tenantId}`],
    { createIfNone: true }
  );

  // TODO(billti) remove proxy hack once proper cors is in place
  const proxyTo = endpointUri;
  endpointUri = "http://localhost:5555";

  const apiVersion = "api-version=2022-09-12-preview";
  // const providerStatusUri = `${endpointUri}${workspaceUri}/providerStatus?${apiVersion}`;
  // const storageSasUri = `${endpointUri}${workspaceUri}/storage/sasUri?${apiVersion}`;
  const quotasUri = `${endpointUri}${workspaceUri}/quotas?${apiVersion}`;

  try {
    const response = await fetch(quotasUri, {
      headers: [
        ["Authorization", `Bearer ${workspaceAuth.accessToken}`],
        ["Content-Type", "application/json"],
        ["x-proxy-to", proxyTo], // TODO(billti) remove once cors is working
      ],
      method: "GET",
    });
    if (!response.ok) throw "Failed to query workspace";

    const quotasObj = (await response.json()) as ResponseTypes.Quotas;
    if (!quotasObj?.value?.length) throw "No quotas found";

    log.debug(`Quotas: ${JSON.stringify(quotasObj, null, 2)}`);
  } catch (e) {
    log.error("Failed to get quotas from workspace with: ", e);
  }
}
