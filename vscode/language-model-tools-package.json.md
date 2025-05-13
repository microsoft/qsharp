# Q# Language Model Tools

## Get Azure Quantum Jobs

- **Name**: `azure-quantum-get-jobs`
- **Description**: Get a list of recent jobs that have been run by the customer, along with their statuses, in the currently active workspace. Call this when you need to know what jobs have been run recently or need a history of jobs run, for example when a customer asks 'What are my recent jobs?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|

## Get Azure Quantum Job Details

- **Name**: `azure-quantum-get-job`
- **Description**: Get the job information for a customer's job given its id. Call this whenever you need to know information about a specific job, for example when a customer asks 'What is the status of my job?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `job_id` | `string` | Yes | Job's unique identifier. |

## Connect to Azure Quantum Workspace

- **Name**: `azure-quantum-connect-to-workspace`
- **Description**: Starts the UI flow to connect to an existing Azure Quantum Workspace. Call this when the customer does not have an active workspace, and agrees to connect to a workspace.

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|

## Download Azure Quantum Job Results

- **Name**: `azure-quantum-download-job-results`
- **Description**: Download and display the results from a customer's job given its id. Call this when you need to download or display as a histogram the results for a job, for example when a customer asks 'What are the results of my last job?' or 'Can you show me the histogram for this job?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `job_id` | `string` | Yes | Job's unique identifier. |

## Get Azure Quantum Workspaces

- **Name**: `azure-quantum-get-workspaces`
- **Description**: Get a list of workspaces the customer has access to, in the form of workspace ids. Call this when you need to know what workspaces the customer has access to, for example when a customer asks 'What are my workspaces?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|

## Submit to Azure Quantum Target

- **Name**: `azure-quantum-submit-to-target`
- **Description**: Submit the current Q# program to Azure Quantum with the provided information. Call this when you need to submit or run a Q# program with Azure Quantum, for example when a customer asks 'Can you submit this program to Azure?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `job_name` | `string` | Yes | The string to name the created job. |
| `target_id` | `string` | Yes | The ID or name of the target to submit the job to. |
| `number_of_shots` | `number` | Yes | The number of shots to use for the job. |

## Get Active Azure Quantum Workspace

- **Name**: `azure-quantum-get-active-workspace`
- **Description**: Get the id of the active workspace for this conversation. Call this when you need to know what workspace is the active workspace being used in the context of the current conversation, for example when a customer asks 'What is the workspace that's being used?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|

## Set Active Azure Quantum Workspace

- **Name**: `azure-quantum-set-active-workspace`
- **Description**: Set the active workspace for this conversation by id. Call this when you need to set what workspace is the active workspace being used in the context of the current conversation, for example when a customer says 'Please use this workspace for my requests.'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | `string` | Yes | The id of the workspace to set as active. |

## Get Azure Quantum Providers

- **Name**: `azure-quantum-get-providers`
- **Description**: Get a list of hardware providers available to the customer, along with their provided targets. Call this when you need to know what providers or targets are available, for example when a customer asks 'What are the available providers?' or 'What targets do I have available?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|

## Get Azure Quantum Target

- **Name**: `azure-quantum-get-target`
- **Description**: Get the target information for a specified target given its id. Call this whenever you need to know information about a specific target, for example when a customer asks 'What is the status of this target?'

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `target_id` | `string` | Yes | The ID of the target to get. |

## Run Q# Program

- **Name**: `qsharp-run-program`
- **Description**: Executes a Q# program. The path to a .qs file must be specified in the `filePath` parameter. A quantum simulator will be run locally. Q# does not provide any CLI tools, so call this tool whenever you need to execute Q#, instead of running any CLI commands. The `shots` parameter controls the number of times to repeat the simulation. If the number of shots is greater than 1, a histogram will be generated, and the results will be displayed in a dedicated panel.

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `filePath` | `string` | Yes | The absolute path to the .qs file. If this file is part of a project, as defined by being in a folder with a qsharp.json file, the whole project will be compiled as the program. |
| `shots` | `number` | No | The number of times to run the program. Defaults to 1 if not specified. |

## Generate Q# Circuit Diagram

- **Name**: `qsharp-generate-circuit`
- **Description**: Generates a visual circuit diagram for a Q# program. The path to a .qs file must be specified in the `filePath` parameter. This tool will compile the Q# program and generate a quantum circuit diagram that visually represents the quantum operations in the code. The diagram will be displayed in a dedicated circuit panel.

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `filePath` | `string` | Yes | The absolute path to the .qs file. If this file is part of a project, as defined by being in a folder with a qsharp.json file, the whole project will be compiled as the program. |

## Run Q# Resource Estimator

- **Name**: `qsharp-run-resource-estimator`
- **Description**: Runs the quantum resource estimator on a Q# program to calculate the required physical resources. The path to a .qs file must be specified in the `filePath` parameter. This tool will analyze the Q# program and generate estimates of the quantum resources required to run the algorithm, such as the number of qubits, T gates, and other quantum operations. Results will be displayed in a dedicated resource estimator panel.

### Input Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `filePath` | `string` | Yes | The absolute path to the .qs file. If this file is part of a project, as defined by being in a folder with a qsharp.json file, the whole project will be compiled as the program. |
| `qubitTypes` | `array` | No | Array of qubit type labels to use for resource estimation. Allowed values:
- 'qubit_gate_ns_e3': Superconducting/spin qubit with 1e-3 error rate
- 'qubit_gate_ns_e4': Superconducting/spin qubit with 1e-4 error rate
- 'qubit_gate_us_e3': Trapped ion qubit with 1e-3 error rate
- 'qubit_gate_us_e4': Trapped ion qubit with 1e-4 error rate
- 'qubit_maj_ns_e4 + surface_code': Majorana qubit with 1e-4 error rate (surface code QEC)
- 'qubit_maj_ns_e6 + surface_code': Majorana qubit with 1e-6 error rate (surface code QEC)
- 'qubit_maj_ns_e4 + floquet_code': Majorana qubit with 1e-4 error rate (floquet code QEC)
- 'qubit_maj_ns_e6 + floquet_code': Majorana qubit with 1e-6 error rate (floquet code QEC) |
| `errorBudget` | `number` | No | Error budget for the resource estimation. Must be a number between 0 and 1. Default is 0.001. |

