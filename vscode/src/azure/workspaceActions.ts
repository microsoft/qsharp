// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp-lang";
import {
  azureRequest,
  AzureUris,
  QuantumUris,
  ResponseTypes,
  storageRequest,
} from "./networkRequests";
import { WorkspaceConnection } from "./treeView";
import {
  shouldExcludeProvider,
  shouldExcludeTarget,
} from "./providerProperties";

export const scopes = {
  armMgmt: "https://management.azure.com/user_impersonation",
  quantum: "https://quantum.microsoft.com/user_impersonation",
};

export async function queryWorkspaces(): Promise<
  WorkspaceConnection | undefined
> {
  log.debug("Querying for account workspaces");
  // *** Authenticate and retrieve tenants the user has Azure resources for ***

  // For the MSA case, you need to query the tenants first and get the underlying AzureAD
  // tenant for the 'guest' MSA. See https://stackoverflow.microsoft.com/a/76246/108570
  const firstAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.armMgmt],
    { createIfNone: true }
  );
  if (!firstAuth) {
    log.error("No authentication session returned");
    return;
  }

  log.trace(`Got first token: ${JSON.stringify(firstAuth, null, 2)}`);
  const firstToken = firstAuth.accessToken;

  const azureUris = new AzureUris();

  const tenants: ResponseTypes.TenantList = await azureRequest(
    azureUris.tenants(),
    firstToken
  );
  log.trace(`Got tenants: ${JSON.stringify(tenants, null, 2)}`);
  if (!tenants?.value?.length) {
    log.error("No tenants returned");
    vscode.window.showErrorMessage(
      "There a no tenants listed for the account. Ensure the account has an Azure subscription."
    );
    return;
  }

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
    if (!tenantAuth) {
      // The user may have cancelled the login
      log.debug("No AAD authentication session returned during 2nd auth");
      return;
    }
    log.trace(`Got tenant token: ${JSON.stringify(tenantAuth, null, 2)}`);
  }
  const tenantToken = tenantAuth.accessToken;

  const subs: ResponseTypes.SubscriptionList = await azureRequest(
    azureUris.subscriptions(),
    tenantToken
  );
  log.trace(`Got subscriptions: ${JSON.stringify(subs, null, 2)}`);
  if (!subs?.value?.length) {
    log.info("No subscriptions returned for the AAD account and tenant");
    vscode.window.showErrorMessage(
      "No Azure subscriptions found for the account and tenant"
    );
    return;
  }

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
    if (!choice) return; // User probably cancelled
    subId = choice.detail;
  }

  // *** Fetch the Quantum Workspaces in the subscription ***
  const workspaces: ResponseTypes.WorkspaceList = await azureRequest(
    azureUris.workspaces(subId),
    tenantToken
  );
  if (log.getLogLevel() >= 5) {
    log.trace(`Got workspaces: ${JSON.stringify(workspaces, null, 2)}`);
  }
  if (!workspaces.value.length) {
    log.info("No workspaces returned for the subscription");
    vscode.window.showErrorMessage(
      "No Quantum Workspaces found in the Azure subscription"
    );
    return;
  }

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
    providers: workspace.properties.providers.map((provider) => ({
      providerId: provider.providerId,
      currentAvailability:
        provider.provisioningState === "Succeeded"
          ? "Available"
          : "Unavailable",
      targets: [], // Will be populated by a later query
    })),
    jobs: [],
  };
  if (log.getLogLevel() >= 5) {
    log.trace(`Workspace object: ${JSON.stringify(result, null, 2)}`);
  }

  return result;
}

export async function getTokenForWorkspace(workspace: WorkspaceConnection) {
  const workspaceAuth = await vscode.authentication.getSession(
    "microsoft",
    [scopes.quantum, `VSCODE_TENANT:${workspace.tenantId}`],
    { createIfNone: true }
  );
  log.trace(`Got workspace token: ${JSON.stringify(workspaceAuth, null, 2)}`);
  return workspaceAuth.accessToken;
}

// Reference for existing queries in Python SDK and Azure schema:
// - https://github.com/microsoft/qdk-python/blob/main/azure-quantum/azure/quantum/_client/aio/operations/_operations.py
// - https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/data-plane/Microsoft.Quantum/preview/2022-09-12-preview/quantum.json
export async function queryWorkspace(workspace: WorkspaceConnection) {
  const token = await getTokenForWorkspace(workspace);

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  const providerStatus: ResponseTypes.ProviderStatusList = await azureRequest(
    quantumUris.providerStatus(),
    token
  );
  if (log.getLogLevel() >= 5) {
    log.trace(
      `Got provider status: ${JSON.stringify(providerStatus, null, 2)}`
    );
  }

  // Update the providers with the target list
  workspace.providers = providerStatus.value.map((provider) => {
    return {
      providerId: provider.id,
      currentAvailability: provider.currentAvailability,
      targets: provider.targets.filter(
        (target) => !shouldExcludeTarget(target.id)
      ),
    };
  });

  workspace.providers = workspace.providers.filter(
    (provider) => !shouldExcludeProvider(provider.providerId)
  );

  log.debug("Fetching the jobs for the workspace");
  const jobs: ResponseTypes.JobList = await azureRequest(
    quantumUris.jobs(),
    token
  );
  log.debug(`Query returned ${jobs.value.length} jobs`);

  if (log.getLogLevel() >= 5) {
    log.trace(`Got jobs: ${JSON.stringify(jobs, null, 2)}`);
  }

  if (jobs.nextLink) {
    log.error("Jobs returned a nextLink. This is not supported yet.");
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
): Promise<string> {
  log.debug(`Fetching job file from ${containerName}/${blobName}`);

  const body = JSON.stringify({ containerName, blobName });
  const sasResponse: ResponseTypes.SasUri = await azureRequest(
    quantumUris.sasUri(),
    token,
    "POST",
    body
  );
  const sasUri = decodeURI(sasResponse.sasUri);
  log.trace(`Got SAS URI: ${sasUri}`);

  const file = await storageRequest(sasUri, "GET");
  if (!file) throw "No file returned";
  const blob = await file.text();
  return blob;
}

export async function submitJob(
  token: string,
  quantumUris: QuantumUris,
  qirFile: Uint8Array | string,
  providerId: string,
  target: string
) {
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

  const jobName = await vscode.window.showInputBox({ prompt: "Job name" });

  // validator for the user-provided number of shots input
  const validateShotsInput = (input: string) => {
    const result = parseFloat(input);
    if (isNaN(result) || Math.floor(result) !== result) {
      return "Number of shots must be an integer";
    }
  };

  const numberOfShots =
    (await vscode.window.showInputBox({
      value: "100",
      prompt: "Number of shots",
      validateInput: validateShotsInput,
    })) || "100";

  // abort if the user hits <Esc> during shots entry
  if (numberOfShots === undefined) {
    return;
  }

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
  const containerPutUri = `${storageAccount}/${containerName}?restype=container&${sasTokenRaw}`;
  await storageRequest(containerPutUri, "PUT");

  // Write the input data
  const inputDataUri = `${storageAccount}/${containerName}/inputData?${sasTokenRaw}`;
  await storageRequest(
    inputDataUri,
    "PUT",
    [["x-ms-blob-type", "BlockBlob"]],
    qirFile
  );

  // PUT the job data
  const putJobUri = quantumUris.jobs(containerName);

  const payload = {
    id: containerName,
    name: jobName,
    providerId,
    target,
    itemType: "Job",
    containerUri: sasResponse.sasUri,
    inputDataUri: `${storageAccount}/${containerName}/inputData`,
    inputDataFormat: "qir.v1",
    outputDataFormat: "microsoft.quantum-results.v1",
    inputParams: {
      entryPoint: "ENTRYPOINT__main",
      arguments: [],
      count: parseInt(numberOfShots),
      // TODO: shots as well?
    },
  };
  await azureRequest(putJobUri, token, "PUT", JSON.stringify(payload));

  vscode.window.showInformationMessage(`Job ${jobName} submitted`);

  return containerName; // The jobId
}
