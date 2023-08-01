// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { log } from "qsharp";
import {
  azureRequest,
  scopes,
  AzureUris,
  QuantumUris,
  ResponseTypes,
} from "./azure";
import { get } from "http";

export async function queryWorkspaces() {
  // *** Authenticate and retrieve tenants the user has Azure resources for ***

  // For the MSA case, you need to query the tenants first and get the underlying AzureAD
  // tenant for the 'guest' MSA. See https://stackoverflow.microsoft.com/a/76246/108570
  const firstAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.armMgmt],
    { createIfNone: true }
  );
  log.debug(`Got first token: ${JSON.stringify(firstAuth, null, 2)}`);
  const firstToken = firstAuth.accessToken;

  const azureUris = new AzureUris();

  const tenants: ResponseTypes.TenantList = await azureRequest(
    azureUris.tenants(),
    firstToken
  );
  if (!tenants?.value?.length) throw "No tenants returned";

  // Quick-pick if more than one
  let tenantId = tenants.value[0].tenantId;
  if (tenants.value.length > 1) {
    const pickItems = tenants.value.map((tenant) => ({
      label: tenant.displayName,
      detail: tenant.tenantId,
    }));
    const choice = await vscode.window.showQuickPick(pickItems, {
      title: "Select a tenant",
    });
    if (!choice) return;
    tenantId = choice.detail;
  }

  // *** Sign-in to that tenant and query the subscriptions available for it ***

  // Skip if first token is already for the correct tenant and for AAD.
  let tenantAuth = firstAuth;
  const matchesTenant = tenantAuth.account.id.startsWith(tenantId);
  const accountType = (tenantAuth as any).account?.type || "";
  if (accountType !== "aad" || !matchesTenant) {
    tenantAuth = await vscode.authentication.getSession(
      "microsoft",
      [scopes.armMgmt, `VSCODE_TENANT:${tenantId}`],
      { createIfNone: true }
    );
    log.debug(`Got tenant token: ${JSON.stringify(tenantAuth, null, 2)}`);
  }
  const tenantToken = tenantAuth.accessToken;

  const subs: ResponseTypes.SubscriptionList = await azureRequest(
    azureUris.subscriptions(),
    tenantToken
  );
  if (!subs?.value?.length) throw "No subscriptions returned";

  // Quick-pick if more than one
  let subId = subs.value[0].subscriptionId;
  if (subs.value.length > 1) {
    const pickItems = subs.value.map((sub) => ({
      label: sub.displayName,
      detail: sub.subscriptionId,
    }));
    const choice = await vscode.window.showQuickPick(pickItems, {
      title: "Select a subscription",
    });
    if (!choice) return;
    subId = choice.detail;
  }

  // *** Fetch the Quantum Workspaces in the subscription ***
  const workspaces: ResponseTypes.WorkspaceList = await azureRequest(
    azureUris.workspaces(subId),
    tenantToken
  );
  if (!workspaces.value.length) throw "Failed to get any workspaces";

  // id will be similar to: "/subscriptions/00000000-1111-2222-3333-444444444444/resourceGroups/quantumResourcegroup/providers/Microsoft.Quantum/Workspaces/quantumworkspace1"
  // endpointUri will be like: "https://quantumworkspace1.westus.quantum.azure.com" (but first segment should be removed)

  // Quick-pick if more than one
  let workspace = workspaces.value[0];
  if (workspaces.value.length > 1) {
    const pickItems = workspaces.value.map((worksp) => ({
      label: worksp.name,
      detail: worksp.id,
      selection: worksp,
    }));
    const choice = await vscode.window.showQuickPick(pickItems, {
      title: "Select a workspace",
    });
    if (!choice) return;
    workspace = choice.selection;
  }

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
  const token = workspaceAuth.accessToken;

  const quantumUris = new QuantumUris(endpointUri, workspaceUri);

  const quotas: ResponseTypes.Quotas = await azureRequest(
    quantumUris.quotas(),
    token
  );

  const jobs: ResponseTypes.Jobs = await azureRequest(
    quantumUris.jobs(),
    token
  );

  if (jobs.nextLink) {
    log.error("TODO: Handle pagination");
  }

  if (jobs.value.length === 0) return;
  // TODO: Populate tree view with results
  const job =
    jobs.value.length === 1
      ? jobs.value[0]
      : jobs.value.find(
          (job) => job.id === "073064ed-2a47-11ee-b8e7-010101010000"
        );

  // TODO: Get a SAS token for this job container
  if (!job) return;
  const fileUri = vscode.Uri.parse(job.outputDataUri);
  const [_, container, blob] = fileUri.path.split("/");
  getJobFiles(container, blob, token, quantumUris);
}

async function getJobFiles(
  containerName: string,
  blobName: string,
  token: string,
  quantumUris: QuantumUris
) {
  const body = JSON.stringify({ containerName, blobName });
  const sasResponse: ResponseTypes.SasUri = await azureRequest(
    quantumUris.sasUri(),
    token,
    "POST",
    body
  );
  const sasUri = decodeURI(sasResponse.sasUri);
  log.debug(`Got SAS URI: ${sasUri}`);

  try {
    const file = await fetch(sasUri, {
      headers: [
        ["x-ms-version", "2023-01-03"],
        ["x-ms-date", new Date().toUTCString()],
      ],
    });
    const blob = await file.text();
    log.debug(`Got file of length ${blob.length}`);
  } catch (e) {
    log.error(`Failed to get file: ${e}`);
  }
}
