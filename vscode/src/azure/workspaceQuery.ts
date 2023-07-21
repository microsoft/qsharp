// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp";
import * as vscode from "vscode";

// https://learn.microsoft.com/en-us/rest/api/resources/tenants/list
const TENANTS_URI =
  "https://management.azure.com/tenants?api-version=2020-01-01";

// https://learn.microsoft.com/en-us/rest/api/resources/subscriptions/list
const SUBS_URI =
  "https://management.azure.com/subscriptions?api-version=2020-01-01";

// https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/resource-manager/Microsoft.Quantum/preview/2022-01-10-preview/quantum.json#L221
const WORKSPACE_URI =
  "https://management.azure.com/subscriptions/${subId}/providers/Microsoft.Quantum/workspaces?api-version=2022-01-10-preview";

// The VS Code first-party app is trusted for both the below scopes.
const ARM_SCOPE = "https://management.azure.com/user_impersonation";
const AQW_SCOPE = "https://quantum.microsoft.com/user_impersonation";

export async function queryWorkspaces() {
  // *** Authenticate and retrieve tenants the user has Azure resources for ***

  // For the MSA case, you need to query the tenants first and get the underlying AzureAD
  // tenant for the 'guest' MSA. See https://stackoverflow.microsoft.com/a/76246/108570
  const firstAuth = await vscode.authentication.getSession(
    "microsoft",
    [ARM_SCOPE],
    { createIfNone: true }
  );

  let response = await fetch(TENANTS_URI, {
    headers: [
      ["Authorization", `Bearer ${firstAuth.accessToken}`],
      ["Content-Type", "application/json"],
    ],
    method: "GET",
  });
  if (!response.ok) throw "Failed to get tenants";

  const tenantsObj = (await response.json()) as {
    value: Array<{ id: string; tenantId: string; displayName: string }>;
  };
  if (!tenantsObj?.value?.length) throw "No tenants returned";

  // TODO: Quick-pick if more than one
  const tenantId = tenantsObj.value[0].tenantId;

  // *** Sign-in to that tenant and query the subscriptions available for it ***

  const tenantAuth = await vscode.authentication.getSession(
    "microsoft",
    [ARM_SCOPE, `VSCODE_TENANT:${tenantId}`],
    { createIfNone: true }
  );

  response = await fetch(SUBS_URI, {
    headers: [
      ["Authorization", `Bearer ${tenantAuth.accessToken}`],
      ["Content-Type", "application/json"],
    ],
    method: "GET",
  });
  if (!response.ok) throw "Failed to get subscriptions";

  const subsObj = (await response.json()) as {
    value: Array<{
      id: string;
      subscriptionId: string;
      tenantId: string;
      displayName: string;
    }>;
  };
  if (!subsObj?.value?.length) throw "No subscriptions returned";

  // TODO: Quick-pick if more than one
  const subId = subsObj.value[0].subscriptionId;

  // *** Fetch the Quantum Workspaces in the subscription ***

  const sub_uri = WORKSPACE_URI.replace("${subId}", subId);
  response = await fetch(sub_uri, {
    headers: [
      ["Authorization", `Bearer ${tenantAuth.accessToken}`],
      ["Content-Type", "application/json"],
    ],
    method: "GET",
  });
  if (!response.ok) throw "Failed to get workspaces";
  const workspacesObj = (await response.json()) as {
    value: Array<{
      id: string;
      name: string;
      location: string;
      properties: {
        providers: Array<{
          providerId: string;
          providerSku: string;
          provisioningState: string;
          resourceUsageId: string;
        }>;
        provisioningState: string;
        storageAccount: string;
        endpointUri: string;
      };
    }>;
  };
  if (!workspacesObj.value.length) throw "Failed to get any workspaces";

  // id will be similar to: "/subscriptions/00000000-1111-2222-3333-444444444444/resourceGroups/quantumResourcegroup/providers/Microsoft.Quantum/Workspaces/quantumworkspace1"
  // endpointUri will be like: "https://quantumworkspace1.westus.quantum.azure.com" (but first portion should be removed)

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

type QuotasResponse = {
  value: Array<{
    scope: string;
    providerId: string;
    period: string;
    holds: number;
    utilization: number;
    limit: number;
  }>;
};

// Reference for existing queries in Python SDK and Azure schema:
// - https://github.com/microsoft/qdk-python/blob/main/azure-quantum/azure/quantum/_client/aio/operations/_operations.py
// - https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/data-plane/Microsoft.Quantum/preview/2022-09-12-preview/quantum.json
export async function queryWorkspace(
  endpointUri: string,
  workspaceUri: string,
  tenantId: string
) {
  // U
  const workspaceAuth = await vscode.authentication.getSession(
    "microsoft",
    [AQW_SCOPE, `VSCODE_TENANT:${tenantId}`],
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

    const quotasObj = (await response.json()) as QuotasResponse;
    if (!quotasObj?.value?.length) throw "No quotas found";

    log.debug(`Quotas: ${JSON.stringify(quotasObj, null, 2)}`);
  } catch (e) {
    log.error("Failed to get quotas from workspace with: ", e);
  }
}
