// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

const publicMgmtEndpoint = "https://management.azure.com";

export const uris = {
  tenants(mgmtEndpoint?: string) {
    // https://learn.microsoft.com/en-us/rest/api/resources/tenants/list
    return `${
      mgmtEndpoint || publicMgmtEndpoint
    }/tenants?api-version=2020-01-01`;
  },
  subscriptions(mgmtEndpoint?: string) {
    // https://learn.microsoft.com/en-us/rest/api/resources/subscriptions/list
    return `${
      mgmtEndpoint || publicMgmtEndpoint
    }/subscriptions?api-version=2020-01-01`;
  },
  workspaces(subscriptionId: string, mgmtEndpoint?: string) {
    // https://github.com/Azure/azure-rest-api-specs/blob/main/specification/quantum/resource-manager/Microsoft.Quantum/preview/2022-01-10-preview/quantum.json#L221
    return `${
      mgmtEndpoint || publicMgmtEndpoint
    }/subscriptions/${subscriptionId}/providers/Microsoft.Quantum/workspaces?api-version=2022-01-10-preview`;
  },
};

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

  export type Quotas = {
    value: Array<{
      scope: string;
      providerId: string;
      period: string;
      holds: number;
      utilization: number;
      limit: number;
    }>;
  };
}
