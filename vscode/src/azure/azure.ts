// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { workspace } from "vscode";

const publicMgmtEndpoint = "https://management.azure.com";

export async function azureRequest(
  uri: string,
  token: string,
  method = "GET",
  body?: string
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

    if (!response.ok) throw "Failed"; // TODO: Proper error propogation
    log.debug(`Got response ${response.status} ${response.statusText}`);
    return await response.json();
  } catch (e) {
    log.error(`Failed to fetch ${uri}: ${e}`);
  }
}

// Different enough to above to warrant it's own function
export async function storageRequest(
  uri: string,
  method: string,
  extraHeaders?: [string, string][],
  body?: string | Uint8Array
) {
  const headers: [string, string][] = [
    ["x-ms-version", "2023-01-03"],
    ["x-ms-date", new Date().toUTCString()],
  ];
  const storageProxy: string | undefined = workspace
    .getConfiguration("Q#")
    .get("storageProxy"); // e.g. in settings.json: "Q#.storageProxy": "https://qsx-proxy.azurewebsites.net/api/proxy";

  if (extraHeaders?.length) headers.push(...extraHeaders);
  if (storageProxy) {
    headers.push(["x-proxy-to", uri]);
    log.debug(`Setting x-proxy-to header to ${uri}`);
    uri = storageProxy;
  }
  try {
    log.debug(`Fetching ${uri} with method ${method}`);
    const response = await fetch(uri, { method, headers, body });
    if (!response.ok) {
      throw "Failed"; // TODO: Proper error propogation
    }
    log.debug(`Got response ${response.status} ${response.statusText}`);
    return response;
  } catch (e) {
    log.error(`Failed to fetch ${uri}: ${e}`);
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
    public id: string // e.g. "/subscriptions/00000000-1111-2222-3333-444444444444/resourceGroups/quantumResourcegroup/providers/Microsoft.Quantum/Workspaces/quantumworkspace1"
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
}

export class StorageUris {
  // Note that to user AzureAD auth, you need a token for https://storage.azure.com/user_impersonation
  // See https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-azure-active-directory#use-oauth-access-tokens-for-authentication

  // Here was use a Shared Access Signature. See https://learn.microsoft.com/en-us/rest/api/storageservices/service-sas-examples
  // Appears to be using an account SAS - https://learn.microsoft.com/en-us/rest/api/storageservices/create-account-sas

  // x-ms-date header should be present in format: Sun, 06 Nov 1994 08:49:37 GMT
  // See https://learn.microsoft.com/en-us/rest/api/storageservices/representation-of-date-time-values-in-headers

  readonly apiVersion = "2023-01-03"; // Pass as x-ms-version header (see https://learn.microsoft.com/en-us/rest/api/storageservices/versioning-for-the-azure-storage-services#authorize-requests-by-using-azure-ad-shared-key-or-shared-key-lite)

  // List containers - do we need this? The response is in XML
  // See https://learn.microsoft.com/en-us/rest/api/storageservices/list-containers2?tabs=shared-access-signatures
  getContainers(storageAccount: string, sas: string) {
    return `https://${storageAccount}.blob.core.windows.net/?comp=list&${sas}`;
  }

  // Same URI for PUT, with a status code of 201 if successful
  getContainer(storageAccount: string, container: string, sas: string) {
    return `https://${storageAccount}.blob.core.windows.net/${container}?restype=container&${sas}`;
  }

  // Same for DELETE, with a status code of 202 if successful
  getBlob(
    storageAccount: string,
    container: string,
    blob: string,
    sas: string
  ) {
    return `https://${storageAccount}.blob.core.windows.net/${container}/${blob}?${sas}`;
  }
  /*
  Same URI as above for put, but must include the following headers:
  - x-ms-blob-type: BlockBlob
  - Content-Length: <n>
  It will return 201 if created.
  */
}

export const scopes = {
  // The VS Code first-party app is trusted for both the below scopes.
  armMgmt: "https://management.azure.com/user_impersonation",
  quantum: "https://quantum.microsoft.com/user_impersonation",
};

// Put all the Response types in a namespace for easy importing

// eslint-disable-next-line @typescript-eslint/no-namespace
export namespace ResponseTypes {
  export type TenantList = {
    value: Array<{ id: string; tenantId: string; displayName: string }>;
  };

  export type SubscriptionList = {
    value: Array<{
      id: string;
      subscriptionId: string;
      tenantId: string;
      displayName: string;
    }>;
  };

  export type WorkspaceList = {
    value: Array<{
      id: string;
      name: string;
      location: string;
      properties: {
        providers: Array<{
          providerId: string; // e.g., 'ionq', 'quantinuum', 'rigetti'
          providerSku: string;
          provisioningState: string; // e.g. 'Succeeded'
          resourceUsageId: string;
        }>;
        provisioningState: string;
        storageAccount: string; // e.g. "/subscriptions/<guid>/resourceGroups/<name>/providers/Microsoft.Storage/storageAccounts/<id>"
        endpointUri: string; // e.g. "https://<workspace-name>.westus.quantum.azure.com". Note: workspace-name should be removed.
      };
    }>;
  };

  export type Quotas = {
    nextLink: string | null;
    value: Array<{
      scope: string;
      providerId: string;
      period: string;
      holds: number;
      utilization: number;
      limit: number;
    }>;
  };

  export type ProviderStatus = {
    nextLink: string | null;
    value: Array<{
      id: string; // ionq, quantinuum, rigetti, etc.
      currentAvailability: "Available" | "Degraded" | "Unavailable";
      targets: Array<{
        id: string; // ionq.qpu, ionq.simulator, rigetti.sim.qvm, quantinuum.sim.h1-2e, etc.
        currentAvailability: "Available" | "Degraded" | "Unavailable";
        averageQueueTime: number; // minutes
        statusPage: string; // url
      }>;
    }>;
  };

  export type Jobs = {
    nextLink: string | null;
    value: Array<{
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
    }>;
  };

  export type SasUri = {
    sasUri: string;
  };
}
