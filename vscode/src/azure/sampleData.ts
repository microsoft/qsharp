// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { WorkspaceConnection } from "./workspaceTree";

export const sampleWorkspace: WorkspaceConnection = {
  id: "/foo/bar",
  name: "Chemistry",
  endpointUri: "https://foo.bar.baz",
  tenantId: "",
  connection: "PAT",
  storageAccount: "",
  targets: [{ providerId: "IonQ", provisioningState: "Succeeded" }],
  jobs: [{ id: "hydrogen-2", status: "Waiting", outputDataUri: "/test/data" }],
};

export const sampleResult = `# Job results for provider IonQ on 2023-06-23 10::34 UTC

START
METADATA\tmetadata1_name_only
METADATA\tmetadata2_name\tmetadata2_value
METADATA\tmetadata3_name\tmetadata3_value
OUTPUT\tTUPLE\t2\t0_t
OUTPUT\tRESULT\t0\t1_t0r
OUTPUT\tDOUBLE\t0.42\t2_t1d
END\t0
START
METADATA\tmetadata1_name_only
METADATA\tmetadata2_name\tmetadata2_value
METADATA\tmetadata3_name\tmetadata3_value
OUTPUT\tTUPLE\t2\t0_t
OUTPUT\tRESULT\t1\t1_t0r
OUTPUT\tDOUBLE\t0.42\t2_t1d
END\t0
START
METADATA\tmetadata1_name_only
METADATA\tmetadata2_name\tmetadata2_value
METADATA\tmetadata3_name\tmetadata3_value
OUTPUT\tTUPLE\t2\t0_t
OUTPUT\tRESULT\t0\t1_t0r
OUTPUT\tDOUBLE\t0.25\t2_t1d
END\t0`;
