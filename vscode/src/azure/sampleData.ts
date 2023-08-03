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
  targets: [
    {
      providerId: "IonQ Aria",
      provisioningState: "Succeeded",
      status: "Online",
      queueTime: 23,
    },
    {
      providerId: "Quantinuum H2-1",
      provisioningState: "Succeeded",
      status: "Online",
      queueTime: 183,
    },
    {
      providerId: "Rigetti Aspen M-3",
      provisioningState: "Succeeded",
      status: "Online",
      queueTime: 5,
    },
  ],
  jobs: [
    {
      id: "abc123",
      name: "hydrogen-2",
      status: "Waiting",
      creationTime: "2023-07-24T17:25:09.1309979Z",
      outputDataUri: "/test/data",
      target: "IonQ",
    },
    {
      id: "abc124",
      name: "hydrogen-1",
      status: "Executing",
      creationTime: "2023-07-24T17:25:07.1309979Z",
      beginExecutionTime: "2023-07-24T17:55:34.1309979Z",
      outputDataUri: "/test/data",
      target: "IonQ",
    },
    {
      id: "abc125",
      name: "hydrogen-test",
      status: "Succeeded",
      creationTime: "2023-07-24T17:00:09.1309979Z",
      beginExecutionTime: "2023-07-24T17:25:34.1309979Z",
      endExecutionTime: "2023-07-24T17:55:34.1309979Z",
      outputDataUri: "/test/data",
      target: "IonQ",
      costEstimate: "$25.34",
    },
    {
      id: "abc127",
      name: "LK-99b",
      status: "Succeeded",
      creationTime: "2023-07-20T12:20:00.1309979Z",
      beginExecutionTime: "2023-07-24T17:25:34.1309979Z",
      endExecutionTime: "2023-07-24T17:55:34.1309979Z",
      outputDataUri: "/test/data",
      target: "IonQ",
      costEstimate: "$745.00",
    },
    {
      id: "abc126",
      name: "LK-99a",
      status: "Failed",
      creationTime: "2023-07-20T17:25:09.1309979Z",
      beginExecutionTime: "2023-07-24T17:25:34.1309979Z",
      endExecutionTime: "2023-07-24T17:55:34.1309979Z",
      outputDataUri: "/test/data",
      target: "IonQ",
      costEstimate: "$25.34",
    },
  ],
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
