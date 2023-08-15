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
  storageRequest,
} from "./azure";
import { WorkspaceConnection } from "./workspaceTree";
import { getResourcePath } from "../extension";

export async function queryWorkspaces(): Promise<
  WorkspaceConnection | undefined
> {
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
  const fixedEndpoint =
    workspace.properties.endpointUri?.replace(
      `https://${workspace.name}.`,
      "https://"
    ) || "";

  const result: WorkspaceConnection = {
    id: workspace.id,
    name: workspace.name,
    endpointUri: fixedEndpoint,
    tenantId,
    connection: "AAD", // TODO
    storageAccount: workspace.properties.storageAccount,
    targets: workspace.properties.providers.map((provider) => ({
      providerId: provider.providerId,
      provisioningState: provider.provisioningState,
    })),
    jobs: [],
  };

  // *** Query the workspace for its properties ***
  await queryWorkspace(result);
  return result;
}

export async function getTokenForWorkspace(workspace: WorkspaceConnection) {
  const workspaceAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.quantum, `VSCODE_TENANT:${workspace.tenantId}`],
    { createIfNone: true }
  );
  return workspaceAuth.accessToken;
}

// Reference for existing queries in Python SDK and Azure schema:
// - https://github.com/microsoft/qdk-python/blob/main/azure-quantum/azure/quantum/_client/aio/operations/_operations.py
// - https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/data-plane/Microsoft.Quantum/preview/2022-09-12-preview/quantum.json
export async function queryWorkspace(workspace: WorkspaceConnection) {
  const token = await getTokenForWorkspace(workspace);

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

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

  // Sort by creation time from newest to oldest
  workspace.jobs = jobs.value
    .sort((a, b) => (a.creationTime < b.creationTime ? 1 : -1))
    .map((job) => ({ ...job }));

  return;
}

export async function getJobFiles(
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
    const file = await storageRequest(sasUri, "GET");
    if (!file) throw "No file returned";
    const blob = await file.text();
    return blob;
  } catch (e) {
    if ((e as any).name === "TypeError") {
      vscode.window.showErrorMessage(
        "Unable to download the file. This could be due to cors issues on the storage account. " +
          "Please allow GET and PUT requests from all origins on the storage account for this workspace. " +
          "See https://go.microsoft.com/fwlink/?LinkId=2221130 for more info."
      );
    }
    log.error(`Failed to get file: ${e}`);
    return "";
  }
}

export async function submitJob(token: string, quantumUris: QuantumUris) {
  // Generate a unique container id of the form "job-<uuid>"
  const id = crypto.getRandomValues(new Uint8Array(16));
  const idChars = Array.from(id)
    .map((b) => b.toString(16))
    .join("");
  // Guid format such as "job-00000000-1111-2222-3333-444444444444"
  const containerName =
    "job-" +
    idChars.substring(0, 8) +
    "-" +
    idChars.substring(8, 12) +
    "-" +
    idChars.substring(12, 16) +
    "-" +
    idChars.substring(16, 20) +
    "-" +
    idChars.substring(20, 32);

  // Get a sasUri for the container
  const body = JSON.stringify({ containerName });
  const sasResponse: ResponseTypes.SasUri = await azureRequest(
    quantumUris.sasUri(),
    token,
    "POST",
    body
  );
  const sasUri = decodeURI(sasResponse.sasUri);

  // Parse the Uri to get the storage account and sasToken
  const sasUriObj = vscode.Uri.parse(sasUri);
  const storageAccount = sasUriObj.scheme + "://" + sasUriObj.authority;

  // Get the raw value to append to other query strings
  const sasTokenRaw = sasResponse.sasUri.substring(
    sasResponse.sasUri.indexOf("?") + 1
  );

  // Create the container
  /*
PUT https://{{BLOB_ENDPOINT}}/{{BLOB_CONTAINER}}?restype=container&{{BLOB_SASPARAMS}}
x-ms-version: 2023-01-03
x-ms-date: {{$datetime rfc1123}}
  */
  const containerPutUri = `${storageAccount}/${containerName}?restype=container&${sasTokenRaw}`;
  const containerPutResponse = await storageRequest(containerPutUri, "PUT");
  // TODO: Check for success

  // Write the input data
  /*
// PUT {{InputSasUri.response.body.$.sasUri}}
https://{{BLOB_ENDPOINT}}/{{BLOB_CONTAINER}}/inputData?{{BLOB_SASPARAMS}}
x-ms-version: 2023-01-03
x-ms-date: {{$datetime rfc1123}}
x-ms-blob-type: BlockBlob
Content-Type: application/octet-stream
  */
  // Get the QIR file
  // Get extension path
  const qirFilePath = getResourcePath("inputData-quantinuum.h1-2.bc");
  const qirFile = await vscode.workspace.fs.readFile(qirFilePath);

  const inputDataUri = `${storageAccount}/${containerName}/inputData?${sasTokenRaw}`;
  // TODO: Extra headers on below and file body
  const inputDataResponse = await storageRequest(
    inputDataUri,
    "PUT",
    [
      ["x-ms-blob-type", "BlockBlob"],
      ["Content-Type", "application/octet-stream"],
    ],
    qirFile
  );

  // PUT the job data
  /*
PUT https://{{QUANTUM_ENDPOINT}}/subscriptions/{{QUANTUM_SUBID}}/resourceGroups/{{QUANTUM_RG}}/providers/Microsoft.Quantum/Workspaces/{{QUANTUM_WORKSPACE}}/jobs/{{JOB_ID}}?api-version=2022-09-12-preview
Content-Type: application/json
Authorization: Bearer {{QUANTUM_TOKEN}}

{
    "id": "{{JOB_ID}}}", "name": "{{JOB_NAME}}",
    "providerId": "quantinuum", "target": "quantinuum.sim.h1-2e", "itemType": "Job",
    "containerUri": "{{ContainerSasUri.response.body.$.sasUri}}",
    "inputDataUri": "{{InputSasUri.response.body.$.sasUri}}",
    "inputDataFormat": "qir.v1", "outputDataFormat": "microsoft.quantum-results.v1",
    "inputParams": { "entryPoint": "program__main", "arguments": [], "count": 100 }
}
  */
  const putJobUri = quantumUris.jobs(containerName);

  const jobName = await vscode.window.showInputBox({ prompt: "Job name" });

  // TODO: See if putting the Uris without the sas tokens works
  const payload = {
    id: containerName,
    name: jobName,
    providerId: "quantinuum",
    target: "quantinuum.sim.h1-2e",
    itemType: "Job",
    containerUri: sasResponse.sasUri,
    inputDataUri: `${storageAccount}/${containerName}/inputData`,
    inputDataFormat: "qir.v1",
    outputDataFormat: "microsoft.quantum-results.v1",
    inputParams: {
      entryPoint: "program__main",
      arguments: [],
      count: 100,
    },
  };
  const jobResponse = await azureRequest(
    putJobUri,
    token,
    "PUT",
    JSON.stringify(payload)
  );
}
