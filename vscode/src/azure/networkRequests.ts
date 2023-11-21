// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { EventType, UserFlowStatus, sendTelemetryEvent } from "../telemetry";
import { getRandomGuid } from "../utils";

const publicMgmtEndpoint = "https://management.azure.com";

export const useProxy = true;

export async function azureRequest(
  uri: string,
  token: string,
  correlationId?: string,
  method = "GET",
  body?: string,
) {
  const headers: [string, string][] = [
    ["Authorization", `Bearer ${token}`],
    ["Content-Type", "application/json"],
  ];

  try {
    log.debug(`Fetching ${uri} with method ${method}`);
    const response = await fetch(uri, {
      headers,
      method,
      body,
    });

    if (!response.ok) {
      log.error("Azure request failed", response);
      if (correlationId) {
        sendTelemetryEvent(
          EventType.AzureRequestFailed,
          {
            reason: `request to azure returned code ${response.status}`,
            correlationId,
          },
          {},
        );
      }
      throw Error(`Azure request failed: ${response.statusText}`);
    }

    log.debug(`Got response ${response.status} ${response.statusText}`);
    const result = await response.json();
    log.trace("Response value: ", result);

    return result;
  } catch (e) {
    if (correlationId) {
      sendTelemetryEvent(
        EventType.AzureRequestFailed,
        { reason: `request to azure failed to return`, correlationId },
        {},
      );
    }
    log.error(`Failed to fetch ${uri}: ${e}`);
    throw e;
  }
}

// Different enough to above to warrant it's own function
export async function storageRequest(
  uri: string,
  method: string,
  token?: string,
  proxy?: string,
  extraHeaders?: [string, string][],
  body?: string | Uint8Array,
  correlationId?: string,
) {
  const headers: [string, string][] = [
    ["x-ms-version", "2023-01-03"],
    ["x-ms-date", new Date().toUTCString()],
  ];
  if (token) headers.push(["Authorization", `Bearer ${token}`]);

  if (extraHeaders?.length) headers.push(...extraHeaders);
  if (proxy) {
    log.debug(`Setting x-proxy-to header to ${uri}`);
    headers.push(["x-proxy-to", uri]);
    uri = proxy;
  }
  try {
    log.debug(`Fetching ${uri} with method ${method}`);
    const response = await fetch(uri, { method, headers, body });
    if (!response.ok) {
      log.error("Storage request failed", response);
      if (correlationId) {
        sendTelemetryEvent(
          EventType.StorageRequestFailed,
          {
            reason: `request to storage on azure returned code ${response.status}`,
            correlationId,
          },
          {},
        );
      }
      throw Error(`Storage request failed: ${response.statusText}`);
    }
    log.debug(`Got response ${response.status} ${response.statusText}`);
    return response;
  } catch (e) {
    log.error(`Failed to fetch ${uri}: ${e}`);
    if (correlationId) {
      sendTelemetryEvent(
        EventType.StorageRequestFailed,
        {
          reason: `request to storage on azure failed to return`,
          correlationId,
        },
        {},
      );
    }
    throw e;
  }
}

export class AzureUris {
  readonly apiVersion = "2020-01-01";

  constructor(public mgmtEndpoint = publicMgmtEndpoint) {}

  tenants() {
    // https://learn.microsoft.com/en-us/rest/api/resources/tenants/list
    return `${this.mgmtEndpoint}/tenants?api-version=${this.apiVersion}`;
  }

  subscriptions() {
    // https://learn.microsoft.com/en-us/rest/api/resources/subscriptions/list
    return `${this.mgmtEndpoint}/subscriptions?api-version=${this.apiVersion}`;
  }

  workspaces(subscriptionId: string) {
    // https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/resource-manager/Microsoft.Quantum/preview/2022-01-10-preview/quantum.json#L221
    return `${this.mgmtEndpoint}/subscriptions/${subscriptionId}/providers/Microsoft.Quantum/workspaces?api-version=2022-01-10-preview`;
  }
}

export class QuantumUris {
  readonly apiVersion = "2022-09-12-preview";

  constructor(
    public endpoint: string, // e.g. "https://westus.quantum.azure.com"
    public id: string, // e.g. "/subscriptions/00000000-1111-2222-3333-444444444444/resourceGroups/quantumResourcegroup/providers/Microsoft.Quantum/Workspaces/quantumworkspace1"
  ) {}

  quotas() {
    return `${this.endpoint}${this.id}/quotas?api-version=${this.apiVersion}`;
  }

  providerStatus() {
    return `${this.endpoint}${this.id}/providerStatus?api-version=${this.apiVersion}`;
  }

  jobs(jobId?: string) {
    return !jobId
      ? `${this.endpoint}${this.id}/jobs?api-version=${this.apiVersion}`
      : `${this.endpoint}${this.id}/jobs/${jobId}?api-version=${this.apiVersion}`;
  }

  // Needs to POST an application/json payload such as: {"containerName": "job-073064ed-2a47-11ee-b8e7-010101010000","blobName":"outputData"}
  sasUri() {
    return `${this.endpoint}${this.id}/storage/sasUri?api-version=${this.apiVersion}`;
  }

  storageProxy() {
    return `${this.endpoint}${this.id}/storage/proxy?api-version=${this.apiVersion}`;
  }
}

export class StorageUris {
  // Requests use a Shared Access Signature. See https://learn.microsoft.com/en-us/rest/api/storageservices/service-sas-examples

  // x-ms-date header should be present in format: Sun, 06 Nov 1994 08:49:37 GMT
  // See https://learn.microsoft.com/en-us/rest/api/storageservices/representation-of-date-time-values-in-headers

  readonly apiVersion = "2023-01-03"; // Pass as x-ms-version header (see https://learn.microsoft.com/en-us/rest/api/storageservices/versioning-for-the-azure-storage-services#authorize-requests-by-using-azure-ad-shared-key-or-shared-key-lite)

  // Same for PUT, with a status code of 201 if successful
  getContainer(storageAccount: string, container: string, sas: string) {
    return `https://${storageAccount}.blob.core.windows.net/${container}?restype=container&${sas}`;
  }

  // Same for DELETE, with a status code of 202 if successful
  // Also same URI for PUT, but must include the following headers:
  // - x-ms-blob-type: BlockBlob
  // - Content-Length: <n>
  // It will return 201 if created.
  getBlob(
    storageAccount: string,
    container: string,
    blob: string,
    sas: string,
  ) {
    return `https://${storageAccount}.blob.core.windows.net/${container}/${blob}?${sas}`;
  }
}

export async function checkCorsConfig(token: string, quantumUris: QuantumUris) {
  const correlationId = getRandomGuid();
  sendTelemetryEvent(EventType.CheckCorsStart, { correlationId }, {});

  log.debug("Checking CORS configuration for the workspace");

  // Get a sasUri for a container to check (it's name doesn't matter, CORS is service wide on a storage account)
  const body: any = JSON.stringify({ containerName: "test" });
  const sasResponse: ResponseTypes.SasUri = await azureRequest(
    quantumUris.sasUri(),
    token,
    correlationId,
    "POST",
    body,
  );
  const sasUri = decodeURI(sasResponse.sasUri);

  /*
  The below doesn't appear to work as it looks like CORS is pre-flighting the manual OPTIONS request!
  See https://stackoverflow.com/questions/77108984/manually-pre-flighting-a-cors-request-is-failing-due-to-cors-issues
  for any better solution. Until then, we'll just try a GET request with the headers we need and see if it works.
  It will throw an exception if it fails due to CORS errors, else should just return a 200, or likely a 404.

  // Check if GET and PUT requests to the storage account are allowed
  log.debug("Checking GET requests are allowed");
  const getResponse = await fetch(sasUri, {
    method: "OPTIONS",
    headers: [
      ["Access-Control-Request-Method", "GET"],
      ["Access-Control-Request-Headers", "x-ms-date,x-ms-version"],
    ],
  });
  if (!getResponse.ok) throw Error("CORS preflight request failed");
  log.debug("Checking PUT requests are allowed");
  const putResponse = await fetch(sasUri, {
    method: "OPTIONS",
    headers: [
      ["Access-Control-Request-Method", "PUT"],
      [
        "Access-Control-Request-Headers",
        "x-ms-date,x-ms-version,x-ms-blob-type",
      ],
    ],
  });
  if (!putResponse.ok) throw Error("CORS preflight request failed");
  */
  log.debug("Checking GET requests are allowed");
  // This will throw if it fails the CORS check, but not if it's a 404 or 200
  await fetch(sasUri, {
    method: "GET",
    headers: [
      ["x-ms-date", new Date().toUTCString()],
      ["x-ms-version", "2023-01-03"],
      ["x-ms-blob-type", "BlockBlob"],
    ],
  });
  log.debug("Pre-flighted GET request didn't throw, so CORS seems good");
  sendTelemetryEvent(
    EventType.CheckCorsEnd,
    { correlationId, flowStatus: UserFlowStatus.Succeeded },
    {},
  );
}

export async function compileToBitcode(
  compilerService: string,
  providerId: string,
  qir: string,
) {
  try {
    log.info("Using compiler service at " + compilerService);
    const bitcodeRequest = await fetch(compilerService, {
      method: "POST",
      headers: {
        "Content-Type": "application/octet-stream",
        "x-hardware-target": providerId,
      },
      body: qir,
    });

    if (!bitcodeRequest.ok) {
      log.error("Failed to compile to QIR bitcode", bitcodeRequest);
      throw Error("Failed to compile to QIR bitcode");
    }
    return new Uint8Array(await bitcodeRequest.arrayBuffer());
  } catch (e) {
    log.error("Failed to compile to QIR bitcode", e);
    throw Error("Failed to compile to QIR bitcode");
  }
}

// Put all the Response types in a namespace for easy importing

// eslint-disable-next-line @typescript-eslint/no-namespace
export namespace ResponseTypes {
  export type Tenant = {
    id: string;
    tenantId: string;
    displayName: string;
  };

  export type TenantList = {
    value: Array<Tenant>;
  };

  export type Subscription = {
    id: string;
    subscriptionId: string;
    tenantId: string;
    displayName: string;
  };

  export type SubscriptionList = {
    value: Array<Subscription>;
  };

  export type Provider = {
    providerId: string;
    providerSku: string;
    provisioningState: string; // e.g. 'Succeeded'
    resourceUsageId: string;
  };

  export type Workspace = {
    id: string;
    name: string;
    location: string;
    properties: {
      providers: Array<Provider>;
      provisioningState: string;
      storageAccount: string; // e.g. "/subscriptions/<guid>/resourceGroups/<name>/providers/Microsoft.Storage/storageAccounts/<id>"
      endpointUri: string; // e.g. "https://<workspace-name>.westus.quantum.azure.com". Note: workspace-name should be removed.
    };
  };

  export type WorkspaceList = {
    value: Array<Workspace>;
  };

  export type Quota = {
    scope: string;
    providerId: string;
    period: string;
    holds: number;
    utilization: number;
    limit: number;
  };

  export type QuotaList = {
    nextLink: string | null;
    value: Array<Quota>;
  };

  export type Target = {
    id: string;
    currentAvailability: "Available" | "Degraded" | "Unavailable";
    averageQueueTime: number; // minutes
    statusPage: string; // url
  };

  export type ProviderStatus = {
    id: string;
    currentAvailability: "Available" | "Degraded" | "Unavailable";
    targets: Array<Target>;
  };

  export type ProviderStatusList = {
    nextLink: string | null;
    value: Array<ProviderStatus>;
  };

  export type Job = {
    id: string;
    name: string;
    target: string;
    containerUri: string;
    inputDataUri: string;
    outputDataUri: string;
    inputDataFormat: string;
    outputDataFormat: string;
    inputParams: any;
    status: "Waiting" | "Executing" | "Succeeded" | "Failed" | "Cancelled";
    creationTime: string;
    beginExecutionTime: string;
    endExecutionTime: string;
    cancellationTime?: string;
    costEstimate?: any;
    errorData?: any;
  };

  export type JobList = {
    nextLink: string | null;
    value: Array<Job>;
  };

  export type SasUri = {
    sasUri: string;
  };
}
