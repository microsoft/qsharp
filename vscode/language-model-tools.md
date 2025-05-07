# Q# Language Model Tools

## Get Azure Quantum Jobs

- **Name**: `azure-quantum-get-jobs`
- **Description**: Get a list of recent jobs that have been run by the customer, along with their statuses, in the currently active workspace. Call this when you need to know what jobs have been run recently or need a history of jobs run, for example when a customer asks 'What are my recent jobs?'

## Get Azure Quantum Job Details

- **Name**: `azure-quantum-get-job`
- **Description**: Get the job information for a customer's job given its id. Call this whenever you need to know information about a specific job, for example when a customer asks 'What is the status of my job?'

## Connect to Azure Quantum Workspace

- **Name**: `azure-quantum-connect-to-workspace`
- **Description**: Starts the UI flow to connect to an existing Azure Quantum Workspace. Call this when the customer does not have an active workspace, and agrees to connect to a workspace.

## Download Azure Quantum Job Results

- **Name**: `azure-quantum-download-job-results`
- **Description**: Download and display the results from a customer's job given its id. Call this when you need to download or display as a histogram the results for a job, for example when a customer asks 'What are the results of my last job?' or 'Can you show me the histogram for this job?'

## Get Azure Quantum Workspaces

- **Name**: `azure-quantum-get-workspaces`
- **Description**: Get a list of workspaces the customer has access to, in the form of workspace ids. Call this when you need to know what workspaces the customer has access to, for example when a customer asks 'What are my workspaces?'

## Submit to Azure Quantum Target

- **Name**: `azure-quantum-submit-to-target`
- **Description**: Submit the current Q# program to Azure Quantum with the provided information. Call this when you need to submit or run a Q# program with Azure Quantum, for example when a customer asks 'Can you submit this program to Azure?'

## Get Active Azure Quantum Workspace

- **Name**: `azure-quantum-get-active-workspace`
- **Description**: Get the id of the active workspace for this conversation. Call this when you need to know what workspace is the active workspace being used in the context of the current conversation, for example when a customer asks 'What is the workspace that's being used?'

## Set Active Azure Quantum Workspace

- **Name**: `azure-quantum-set-active-workspace`
- **Description**: Set the active workspace for this conversation by id. Call this when you need to set what workspace is the active workspace being used in the context of the current conversation, for example when a customer says 'Please use this workspace for my requests.'

## Get Azure Quantum Providers

- **Name**: `azure-quantum-get-providers`
- **Description**: Get a list of hardware providers available to the customer, along with their provided targets. Call this when you need to know what providers or targets are available, for example when a customer asks 'What are the available providers?' or 'What targets do I have available?'

## Get Azure Quantum Target

- **Name**: `azure-quantum-get-target`
- **Description**: Get the target information for a specified target given its id. Call this whenever you need to know information about a specific target, for example when a customer asks 'What is the status of this target?'

## Run Q# Program

- **Name**: `qsharp-run-program`
- **Description**: Run the current Q# program in the editor locally. Call this when the user wants to execute the Q# program currently open in the editor, for example when they ask 'Can you run this program?'

