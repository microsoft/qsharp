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
  useProxy,
} from "./networkRequests";
import { WorkspaceConnection } from "./treeView";
import {
  shouldExcludeProvider,
  shouldExcludeTarget,
} from "./providerProperties";
import { getRandomGuid } from "../utils";
import { EventType, sendTelemetryEvent, UserFlowStatus } from "../telemetry";

export const scopes = {
  armMgmt: "https://management.azure.com/user_impersonation",
  quantum: "https://quantum.microsoft.com/user_impersonation",
};

export async function getAuthSession(
  scopes: string[],
  correlationId: string,
): Promise<vscode.AuthenticationSession> {
  sendTelemetryEvent(EventType.AuthSessionStart, { correlationId }, {});
  log.debug("About to getSession for scopes", scopes.join(","));
  try {
    let session = await vscode.authentication.getSession("microsoft", scopes, {
      silent: true,
    });
    if (!session) {
      log.debug("No session with silent request. Trying with createIfNone");
      session = await vscode.authentication.getSession("microsoft", scopes, {
        createIfNone: true,
      });
    }
    log.debug("Got session: ", JSON.stringify(session, null, 2));
    sendTelemetryEvent(
      EventType.AuthSessionEnd,
      {
        correlationId,
        flowStatus: UserFlowStatus.Succeeded,
      },
      {},
    );
    return session;
  } catch (e) {
    sendTelemetryEvent(
      EventType.AuthSessionEnd,
      {
        correlationId,
        reason: "exception in getAuthSession",
        flowStatus: UserFlowStatus.Failed,
      },
      {},
    );
    log.error("Exception occurred in getAuthSession: ", e);
    throw e;
  }
}

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

export async function queryWorkspaces(): Promise<
  WorkspaceConnection | undefined
> {
  log.debug("Querying for account workspaces");
  const correlationId = getRandomGuid();
  sendTelemetryEvent(EventType.QueryWorkspacesStart, { correlationId }, {});
  // *** Authenticate and retrieve tenants the user has Azure resources for ***

  // For the MSA case, you need to query the tenants first and get the underlying AzureAD
  // tenant for the 'guest' MSA. See https://stackoverflow.microsoft.com/a/76246/108570
  const firstAuth = await getAuthSession([scopes.armMgmt], correlationId);

  if (!firstAuth) {
    log.error("No authentication session returned");
    sendTelemetryEvent(
      EventType.QueryWorkspacesEnd,
      {
        correlationId,
        reason: "no auth session returned",
        flowStatus: UserFlowStatus.Failed,
      },
      {},
    );
    return;
  }

  const firstToken = firstAuth.accessToken;
  const azureUris = new AzureUris();

  const tenants: ResponseTypes.TenantList = await azureRequest(
    azureUris.tenants(),
    firstToken,
    correlationId,
  );
  log.trace(`Got tenants: ${JSON.stringify(tenants, null, 2)}`);
  if (!tenants?.value?.length) {
    log.error("No tenants returned");
    sendTelemetryEvent(
      EventType.QueryWorkspacesEnd,
      {
        correlationId,
        reason: "no tenants exist for account",
        flowStatus: UserFlowStatus.Failed,
      },
      {},
    );
    vscode.window.showErrorMessage(
      "There a no tenants listed for the account. Ensure the account has an Azure subscription.",
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
    if (!choice) {
      sendTelemetryEvent(
        EventType.QueryWorkspacesEnd,
        {
          correlationId,
          reason: "aborted tenant choice",
          flowStatus: UserFlowStatus.Aborted,
        },
        {},
      );
      return;
    }
    tenantId = choice.detail;
  }

  // *** Sign-in to that tenant and query the subscriptions available for it ***

  // Skip if first token is already for the correct tenant and for AAD.
  let tenantAuth = firstAuth;
  const matchesTenant = tenantAuth.account.id.startsWith(tenantId);
  const accountType = (tenantAuth as any).account?.type || "";
  if (accountType !== "aad" || !matchesTenant) {
    tenantAuth = await getAuthSession(
      [scopes.armMgmt, `VSCODE_TENANT:${tenantId}`],
      correlationId,
    );
    if (!tenantAuth) {
      sendTelemetryEvent(
        EventType.QueryWorkspacesEnd,
        {
          correlationId,
          reason: "authentication session did not return",
          flowStatus: UserFlowStatus.Aborted,
        },
        {},
      );
      // The user may have cancelled the login
      log.debug("No AAD authentication session returned during 2nd auth");
      return;
    }
  }
  const tenantToken = tenantAuth.accessToken;

  const subs: ResponseTypes.SubscriptionList = await azureRequest(
    azureUris.subscriptions(),
    tenantToken,
    correlationId,
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
          correlationId,
          reason: "aborted subscription choice",
          flowStatus: UserFlowStatus.Aborted,
        },
        {},
      );
      return;
    }
    subId = choice.detail;
  }

  // *** Fetch the Quantum Workspaces in the subscription ***
  const workspaces: ResponseTypes.WorkspaceList = await azureRequest(
    azureUris.workspaces(subId),
    tenantToken,
    correlationId,
  );
  if (log.getLogLevel() >= 5) {
    log.trace(`Got workspaces: ${JSON.stringify(workspaces, null, 2)}`);
  }
  if (!workspaces.value.length) {
    log.info("No workspaces returned for the subscription");
    sendTelemetryEvent(
      EventType.QueryWorkspacesEnd,
      {
        correlationId,
        reason: "no quantum workspaces in azure subscription",
        flowStatus: UserFlowStatus.Aborted,
      },
      {},
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
          correlationId,
          reason: "aborted workspace selection",
          flowStatus: UserFlowStatus.Aborted,
        },
        {},
      );
      return;
    }
    workspace = choice.selection;
  }

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
  const correlationId = getRandomGuid();

  const workspaceAuth = await getAuthSession(
    [scopes.quantum, `VSCODE_TENANT:${workspace.tenantId}`],
    correlationId,
  );
  return workspaceAuth.accessToken;
}

// Reference for existing queries in Python SDK and Azure schema:
// - https://github.com/microsoft/qdk-python/blob/main/azure-quantum/azure/quantum/_client/aio/operations/_operations.py
// - https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/data-plane/Microsoft.Quantum/preview/2022-09-12-preview/quantum.json
export async function queryWorkspace(workspace: WorkspaceConnection) {
  const token = await getTokenForWorkspace(workspace);

  const correlationId = getRandomGuid();
  sendTelemetryEvent(EventType.QueryWorkspaceStart, { correlationId }, {});

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  const providerStatus: ResponseTypes.ProviderStatusList = await azureRequest(
    quantumUris.providerStatus(),
    token,
    correlationId,
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
    correlationId,
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
        correlationId,
        reason: "no jobs returned",
        flowStatus: UserFlowStatus.Aborted,
      },
      {},
    );
    return;
  }

  // Sort by creation time from newest to oldest
  workspace.jobs = jobs.value
    .sort((a, b) => (a.creationTime < b.creationTime ? 1 : -1))
    .map((job) => ({ ...job }));

  sendTelemetryEvent(
    EventType.QueryWorkspaceEnd,
    { correlationId, flowStatus: UserFlowStatus.Succeeded },
    {},
  );

  return;
}

export async function getJobFiles(
  containerName: string,
  blobName: string,
  token: string,
  quantumUris: QuantumUris,
): Promise<string> {
  const correlationId = getRandomGuid();
  log.debug(`Fetching job file from ${containerName}/${blobName}`);
  sendTelemetryEvent(EventType.GetJobFilesStart, { correlationId }, {});

  const body = JSON.stringify({ containerName, blobName });
  const sasResponse: ResponseTypes.SasUri = await azureRequest(
    quantumUris.sasUri(),
    token,
    correlationId,
    "POST",
    body,
  );
  const sasUri = decodeURI(sasResponse.sasUri);
  log.trace(`Got SAS URI: ${sasUri}`);

  const file = await storageRequest(
    sasUri,
    "GET",
    useProxy ? token : undefined,
    useProxy ? quantumUris.storageProxy() : undefined,
  );

  if (!file) {
    sendTelemetryEvent(
      EventType.GetJobFilesEnd,
      {
        correlationId,
        reason: "no files returned",
        flowStatus: UserFlowStatus.Aborted,
      },
      {},
    );
    throw "No file returned";
  }

  const blob = await file.text();
  sendTelemetryEvent(
    EventType.GetJobFilesEnd,
    { correlationId, flowStatus: UserFlowStatus.Succeeded },
    {},
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
  const correlationId = getRandomGuid();
  sendTelemetryEvent(EventType.SubmitToAzureStart, { correlationId }, {});

  const containerName = getRandomGuid();
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
    sendTelemetryEvent(
      EventType.SubmitToAzureEnd,
      {
        correlationId,
        reason: "undefined number of shots",
        flowStatus: UserFlowStatus.Aborted,
      },
      {},
    );
    return;
  }

  // Get a sasUri for the container
  const body = JSON.stringify({ containerName });
  const sasResponse = await azureRequest(
    quantumUris.sasUri(),
    token,
    correlationId,
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
    useProxy ? token : undefined,
    useProxy ? quantumUris.storageProxy() : undefined,
    undefined,
    undefined,
    correlationId,
  );

  // Write the input data
  const inputDataUri = `${storageAccount}/${containerName}/inputData?${sasTokenRaw}`;
  await storageRequest(
    inputDataUri,
    "PUT",
    useProxy ? token : undefined,
    useProxy ? quantumUris.storageProxy() : undefined,
    [
      ["x-ms-blob-type", "BlockBlob"],
      ["Content-Type", "text/plain"],
    ],
    qirFile,
    correlationId,
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
    correlationId,
    "PUT",
    JSON.stringify(payload),
  );

  vscode.window.showInformationMessage(`Job ${jobName} submitted`);
  sendTelemetryEvent(
    EventType.SubmitToAzureEnd,
    {
      correlationId,
      reason: "job submitted",
      flowStatus: UserFlowStatus.Succeeded,
    },
    {},
  );

  return containerName; // The jobId
}
