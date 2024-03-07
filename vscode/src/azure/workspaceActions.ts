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
import { getRandomGuid } from "../utils";
import { EventType, sendTelemetryEvent, UserFlowStatus } from "../telemetry";
import { getTenantIdAndToken, getTokenForWorkspace } from "./auth";

export function getAzurePortalWorkspaceLink(workspace: WorkspaceConnection) {
  // Portal link format:
  // - https://portal.azure.com/#resource/subscriptions/<sub guid>/resourceGroups/<group>/providers/Microsoft.Quantum/Workspaces/<name>/overview

  return `https://portal.azure.com/#resource${workspace.id}/overview`;
}

export function getPythonCodeForWorkspace(
  id: string,
  endpointUri: string,
  name: string,
) {
  // id starts with the pattern: "/subscriptions/<sub guid>/resourceGroups/<group>>/providers/Microsoft.Quantum/Workspaces/<name>"
  // endpointUri format: "https:/westus2.quantum.azure.com"

  // Regular expression to extract subscriptionId and resourceGroup from the id
  const idRegex =
    /\/subscriptions\/(?<subscriptionId>[^/]+)\/resourceGroups\/(?<resourceGroup>[^/]+)/;

  // Regular expression to extract the first part of the endpointUri
  const endpointRegex = /https:\/\/(?<location>[^.]+)\./;

  const idMatch = id.match(idRegex);
  const endpointMatch = endpointUri.match(endpointRegex);

  const subscriptionId = idMatch?.groups?.subscriptionId;
  const resourceGroup = idMatch?.groups?.resourceGroup;
  const location = endpointMatch?.groups?.location;

  // TODO: Mention how to fetch/use connection strings

  const pythonCode = `# If developing locally, on first run this will open a browser to authenticate the
# connection with Azure. In remote scenarios, such as SSH or Codespaces, it may
# be necesssary to install the Azure CLI and run 'az login --use-device-code' to
# authenticate. For unattended scenarios, such as batch jobs, a service principal
# should be configured and used for authentication. For more information, see
# https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication-overview

import azure.quantum

workspace = azure.quantum.Workspace(
    subscription_id = "${subscriptionId || "MY_SUBSCRIPTION_ID"}",
    resource_group = "${resourceGroup || "MY_RESOURCE_GROUP"}",
    name = "${name || "MY_WORKSPACE_NAME"}",
    location = "${location || "MY_WORKSPACE_LOCATION"}",
)
`;

  return pythonCode;
}

async function getWorkspaceWithConnectionString(): Promise<
  WorkspaceConnection | undefined
> {
  // TODO: Telemetry

  const connStr = await vscode.window.showInputBox({
    prompt: "Enter the connection string",
    placeHolder:
      "SubscriptionId=<guid>;ResourceGroupName=<name>;WorkspaceName=<name>;ApiKey=<secret>;QuantumEndpoint=<serviceUri>;",
  });
  if (!connStr) return;

  const parts = connStr.split(";");
  if (!parts) {
    vscode.window.showErrorMessage(
      "Invalid connection string. Please follow the placeholder format",
    );
    return;
  }

  const partsMap = new Map<string, string>();
  parts.forEach((part) => {
    const eq = part.indexOf("=");
    if (eq === -1) return;
    partsMap.set(part.substring(0, eq).toLowerCase(), part.substring(eq + 1));
  });

  if (
    !partsMap.has("subscriptionid") ||
    !partsMap.has("resourcegroupname") ||
    !partsMap.has("workspacename") ||
    !partsMap.has("apikey") ||
    !partsMap.has("quantumendpoint")
  ) {
    vscode.window.showErrorMessage(
      "Invalid connection string. Please follow the placeholder format",
    );
    return;
  }

  const workspaceId =
    `/subscriptions/${partsMap.get("subscriptionid")}` +
    `/resourceGroups/${partsMap.get("resourcegroupname")}` +
    `/providers/Microsoft.Quantum/Workspaces/${partsMap.get("workspacename")}`;

  // TODO: Validate the connection string info before returning as valid for further use.
  // e.g. check for 401 (invalid key), 404 (invalid workspace), endpoint not found (invalid endpoint), etc.

  return {
    id: workspaceId,
    name: partsMap.get("workspacename")!,
    endpointUri: partsMap.get("quantumendpoint")!,
    tenantId: "", // Blank means not authenticated via a token
    apiKey: partsMap.get("apikey"),
    providers: [], // Providers and jobs will be populated by a following 'queryWorkspace' call
    jobs: [],
  };
}

async function chooseWorkspaceFromSubscription(
  subId: string,
  associationId: string,
  token: string,
  start: number,
) {
  // *** Fetch the Quantum Workspaces in the subscription ***
  const workspaces: ResponseTypes.WorkspaceList = await azureRequest(
    AzureUris.workspaces(subId),
    token,
    associationId,
  );
  if (log.getLogLevel() >= 5) {
    log.trace(`Got workspaces: ${JSON.stringify(workspaces, null, 2)}`);
  }
  if (!workspaces.value.length) {
    log.info("No workspaces returned for the subscription");
    sendTelemetryEvent(
      EventType.QueryWorkspacesEnd,
      {
        associationId,
        reason: "no quantum workspaces in azure subscription",
        flowStatus: UserFlowStatus.Aborted,
      },
      { timeToCompleteMs: performance.now() - start },
    );
    vscode.window.showErrorMessage(
      "No Quantum Workspaces found in the Azure subscription",
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
    if (!choice) {
      sendTelemetryEvent(
        EventType.QueryWorkspacesEnd,
        {
          associationId,
          reason: "aborted workspace selection",
          flowStatus: UserFlowStatus.Aborted,
        },
        { timeToCompleteMs: performance.now() - start },
      );
      return;
    }
    workspace = choice.selection;
  }
  return workspace;
}

async function chooseSubscriptionFromTenant(
  tenantToken: string,
  associationId: string,
  start: number,
) {
  const subs: ResponseTypes.SubscriptionList = await azureRequest(
    AzureUris.subscriptions(),
    tenantToken,
    associationId,
  );
  log.trace(`Got subscriptions: ${JSON.stringify(subs, null, 2)}`);
  if (!subs?.value?.length) {
    log.info("No subscriptions returned for the AAD account and tenant");
    vscode.window.showErrorMessage(
      "No Azure subscriptions found for the account and tenant",
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
    if (!choice) {
      // User probably cancelled
      sendTelemetryEvent(
        EventType.QueryWorkspacesEnd,
        {
          associationId,
          reason: "aborted subscription choice",
          flowStatus: UserFlowStatus.Aborted,
        },
        { timeToCompleteMs: performance.now() - start },
      );
      return;
    }
    subId = choice.detail;
  }
  return subId;
}

async function getWorkspaceWithAzureAD(): Promise<
  WorkspaceConnection | undefined
> {
  const associationId = getRandomGuid();
  const start = performance.now();
  sendTelemetryEvent(EventType.QueryWorkspacesStart, { associationId }, {});

  // *** Authenticate and retrieve tenants the user has Azure resources for ***
  const tenantAuth = await getTenantIdAndToken(associationId, start);
  if (!tenantAuth) return;

  const subscriptionId = await chooseSubscriptionFromTenant(
    tenantAuth.tenantToken,
    associationId,
    start,
  );
  if (!subscriptionId) return;

  const workspace = await chooseWorkspaceFromSubscription(
    subscriptionId,
    associationId,
    tenantAuth.tenantToken,
    start,
  );
  if (!workspace) return;

  // Need to remove the first part of the endpoint
  const fixedEndpoint =
    workspace.properties.endpointUri?.replace(
      `https://${workspace.name}.`,
      "https://",
    ) || "";

  const result: WorkspaceConnection = {
    id: workspace.id,
    name: workspace.name,
    endpointUri: fixedEndpoint,
    tenantId: tenantAuth.tenantId,
    providers: [], // Providers and jobs will be populated by a following 'queryWorkspace' call
    jobs: [],
  };
  if (log.getLogLevel() >= 5) {
    log.trace(`Workspace object: ${JSON.stringify(result, null, 2)}`);
  }

  return result;
}

export async function queryWorkspaces(): Promise<
  WorkspaceConnection | undefined
> {
  log.debug("Querying for account workspaces");

  // Show a quick-pick to ask if they want to connect via AzureAD or a connection string
  const choice = await vscode.window.showQuickPick(
    [
      { label: "Azure account", type: "aad" },
      { label: "Connection string", type: "connStr" },
    ],
    { canPickMany: false, title: "Select an authentication method" },
  );

  switch (choice?.type) {
    case "connStr":
      return await getWorkspaceWithConnectionString();
    case "aad":
      return await getWorkspaceWithAzureAD();
    default:
      return;
  }
}

// Reference for existing queries in Python SDK and Azure schema:
// - https://github.com/microsoft/qdk-python/blob/main/azure-quantum/azure/quantum/_client/aio/operations/_operations.py
// - https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/data-plane/Microsoft.Quantum/preview/2022-09-12-preview/quantum.json
export async function queryWorkspace(workspace: WorkspaceConnection) {
  const start = performance.now();
  const token = await getTokenForWorkspace(workspace);

  const associationId = getRandomGuid();
  sendTelemetryEvent(EventType.QueryWorkspaceStart, { associationId }, {});

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  const providerStatus: ResponseTypes.ProviderStatusList = await azureRequest(
    quantumUris.providerStatus(),
    token,
    associationId,
  );
  if (log.getLogLevel() >= 5) {
    log.trace(
      `Got provider status: ${JSON.stringify(providerStatus, null, 2)}`,
    );
  }

  // Update the providers with the target list
  workspace.providers = providerStatus.value.map((provider) => {
    return {
      providerId: provider.id,
      currentAvailability: provider.currentAvailability,
      targets: provider.targets.filter(
        (target) => !shouldExcludeTarget(target.id),
      ),
    };
  });

  workspace.providers = workspace.providers.filter(
    (provider) => !shouldExcludeProvider(provider.providerId),
  );

  log.debug("Fetching the jobs for the workspace");
  const jobs: ResponseTypes.JobList = await azureRequest(
    quantumUris.jobs(),
    token,
    associationId,
  );
  log.debug(`Query returned ${jobs.value.length} jobs`);

  if (log.getLogLevel() >= 5) {
    log.trace(`Got jobs: ${JSON.stringify(jobs, null, 2)}`);
  }

  if (jobs.nextLink) {
    log.error("Jobs returned a nextLink. This is not supported yet.");
  }

  if (jobs.value.length === 0) {
    sendTelemetryEvent(
      EventType.QueryWorkspaceEnd,
      {
        associationId,
        reason: "no jobs returned",
        flowStatus: UserFlowStatus.Aborted,
      },
      { timeToCompleteMs: performance.now() - start },
    );
    return;
  }

  // Sort by creation time from newest to oldest
  workspace.jobs = jobs.value
    .sort((a, b) => (a.creationTime < b.creationTime ? 1 : -1))
    .map((job) => ({ ...job }));

  sendTelemetryEvent(
    EventType.QueryWorkspaceEnd,
    { associationId, flowStatus: UserFlowStatus.Succeeded },
    { timeToCompleteMs: performance.now() - start },
  );

  return;
}

export async function getJobFiles(
  containerName: string,
  blobName: string,
  token: string,
  quantumUris: QuantumUris,
): Promise<string> {
  const start = performance.now();
  const associationId = getRandomGuid();
  log.debug(`Fetching job file from ${containerName}/${blobName}`);
  sendTelemetryEvent(EventType.GetJobFilesStart, { associationId }, {});

  const body = JSON.stringify({ containerName, blobName });
  const sasResponse: ResponseTypes.SasUri = await azureRequest(
    quantumUris.sasUri(),
    token,
    associationId,
    "POST",
    body,
  );
  const sasUri = decodeURI(sasResponse.sasUri);
  log.trace(`Got SAS URI: ${sasUri}`);

  const file = await storageRequest(
    sasUri,
    "GET",
    token,
    quantumUris.storageProxy(),
  );

  if (!file) {
    sendTelemetryEvent(
      EventType.GetJobFilesEnd,
      {
        associationId,
        reason: "no files returned",
        flowStatus: UserFlowStatus.Aborted,
      },
      { timeToCompleteMs: performance.now() - start },
    );
    throw new Error("Storage service did not return a file.");
  }

  const blob = await file.text();
  sendTelemetryEvent(
    EventType.GetJobFilesEnd,
    { associationId, flowStatus: UserFlowStatus.Succeeded },
    { timeToCompleteMs: performance.now() - start },
  );
  return blob;
}

export async function submitJob(
  token: string,
  quantumUris: QuantumUris,
  qirFile: Uint8Array | string,
  providerId: string,
  target: string,
) {
  const associationId = getRandomGuid();
  const start = performance.now();
  sendTelemetryEvent(EventType.SubmitToAzureStart, { associationId }, {});

  const containerName = getRandomGuid();
  const jobName = await vscode.window.showInputBox({
    prompt: "Job name",
    value: new Date().toISOString(),
  });
  if (!jobName) return; // TODO: Log a telemetry event for this?

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
    sendTelemetryEvent(
      EventType.SubmitToAzureEnd,
      {
        associationId,
        reason: "undefined number of shots",
        flowStatus: UserFlowStatus.Aborted,
      },
      { timeToCompleteMs: performance.now() - start },
    );
    return;
  }

  // Get a sasUri for the container
  const body = JSON.stringify({ containerName });
  const sasResponse = await azureRequest(
    quantumUris.sasUri(),
    token,
    associationId,
    "POST",
    body,
  );

  const sasUri = decodeURI(sasResponse.sasUri);

  // Parse the Uri to get the storage account and sasToken
  const sasUriObj = vscode.Uri.parse(sasUri);
  const storageAccount = sasUriObj.scheme + "://" + sasUriObj.authority;

  // Get the raw value to append to other query strings
  const sasTokenRaw = sasResponse.sasUri.substring(
    sasResponse.sasUri.indexOf("?") + 1,
  );

  // Create the container
  const containerPutUri = `${storageAccount}/${containerName}?restype=container&${sasTokenRaw}`;
  await storageRequest(
    containerPutUri,
    "PUT",
    token,
    quantumUris.storageProxy(),
    undefined,
    undefined,
    associationId,
  );

  // Write the input data
  const inputDataUri = `${storageAccount}/${containerName}/inputData?${sasTokenRaw}`;
  await storageRequest(
    inputDataUri,
    "PUT",
    token,
    quantumUris.storageProxy(),
    [
      ["x-ms-blob-type", "BlockBlob"],
      ["Content-Type", "text/plain"],
    ],
    qirFile,
    associationId,
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
      shots: parseInt(numberOfShots),
    },
  };
  await azureRequest(
    putJobUri,
    token,
    associationId,
    "PUT",
    JSON.stringify(payload),
  );

  vscode.window.showInformationMessage(`Job ${jobName} submitted`);
  sendTelemetryEvent(
    EventType.SubmitToAzureEnd,
    {
      associationId,
      reason: "job submitted",
      flowStatus: UserFlowStatus.Succeeded,
    },
    { timeToCompleteMs: performance.now() - start },
  );

  return containerName; // The jobId
}
