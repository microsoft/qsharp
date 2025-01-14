# TODO

- Get a table response for 'last 5 jobs tagged carbon'
- Have a button that fetches the histogram for the job
- Add an embedding for a 'histogram' widget and see if it renders

# Test scripts

## Environments:

- No workspaces added.
- One workspace
- More than one workspace

## Scenarios:

- user: "show me my jobs"
  - assistant: Should pick up on the fact that there are no workspaces, and should prompt for adding one.

Unsuccessful conversation:

Problems:

- "Let's retrieve your recent jobs" should have been a tool call
- "anyway can you submit this program to azure quantum?\n" should have provoked a tool call

```
[
  {
    "role": "user",
    "content": "show me my jobs\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "id": "3a199106-2992-4166-886b-1fd9ba6faa0d",
        "name": "GetJobs",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"error\":\"There are no Azure Quantum workspace connections set up.\"}",
    "toolCallId": "3a199106-2992-4166-886b-1fd9ba6faa0d"
  },
  {
    "role": "assistant",
    "content": "It looks like you don't have any Azure Quantum workspace connections set up. Would you like to connect to an existing Azure Quantum Workspace?"
  },
  {
    "role": "user",
    "content": "yeah\n"
  },
  {
    "role": "assistant",
    "content": "Please follow the UI flow to connect to your existing Azure Quantum Workspace. Once connected, you can view your jobs and manage your quantum tasks.",
    "ToolCalls": [
      {
        "id": "f334b359-289d-4171-986e-cf5ceb0de992",
        "name": "ConnectToWorkspace",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"error\":\"There are no Azure Quantum workspace connections set up.\"}",
    "toolCallId": "f334b359-289d-4171-986e-cf5ceb0de992"
  },
  {
    "role": "assistant",
    "content": "It seems there was an issue connecting to your Azure Quantum Workspace. Please ensure you have an existing workspace set up in your Azure account. If you need assistance, you can refer to the [Azure Quantum documentation](https://learn.microsoft.com/en-us/azure/quantum/)."
  },
  {
    "role": "user",
    "content": "try connecting again\n"
  },
  {
    "role": "assistant",
    "content": "Please follow the UI flow to connect to your existing Azure Quantum Workspace. Once connected, you can view your jobs and manage your quantum tasks.",
    "ToolCalls": [
      {
        "id": "3810a77a-7dc0-4737-9f2f-e823acb06886",
        "name": "ConnectToWorkspace",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"result\":\"Connected to Azure Quantum Workspace: `New-Quantum`\"}",
    "toolCallId": "3810a77a-7dc0-4737-9f2f-e823acb06886"
  },
  {
    "role": "assistant",
    "content": "You are now connected to the Azure Quantum Workspace: `New-Quantum`. You can now view your jobs and manage your quantum tasks. Would you like to see your recent jobs?"
  },
  {
    "role": "user",
    "content": "yep!\n"
  },
  {
    "role": "assistant",
    "content": "Let's retrieve your recent jobs.\n\n"
  },
  {
    "role": "user",
    "content": "yes retrieve them please!\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "id": "3d962183-2dba-4aae-86b7-a30947f7022a",
        "name": "GetJobs",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"recentJobs\":[]}",
    "toolCallId": "3d962183-2dba-4aae-86b7-a30947f7022a"
  },
  {
    "role": "assistant",
    "content": "It looks like there are no recent jobs in your Azure Quantum Workspace. If you need assistance with submitting a job or any other task, feel free to ask!"
  },
  {
    "role": "user",
    "content": "can you retrieve older jobs?\n"
  },
  {
    "role": "assistant",
    "content": "Currently, I can only retrieve recent jobs from your Azure Quantum Workspace. If you need to access older jobs, you might want to check the Azure Quantum portal directly. If you need help with anything else, please let me know!"
  },
  {
    "role": "user",
    "content": "where's the azure quantum portal?\n"
  },
  {
    "role": "assistant",
    "content": "You can access the Azure Quantum portal [here](https://portal.azure.com/). From there, you can manage your quantum workspaces, view job history, and access other Azure Quantum services."
  },
  {
    "role": "user",
    "content": "anyway can you submit this program to azure quantum?\n"
  },
  {
    "role": "assistant",
    "content": "Sure, please provide the Q# program you would like to submit, and I'll help you with the submission process."
  },
  {
    "role": "user",
    "content": "this one right here\n"
  },
  {
    "role": "assistant",
    "content": "Please paste the Q# program code here, and I'll assist you with submitting it to Azure Quantum."
  },
  {
    "role": "user",
    "content": "`use q = Qubit()`\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "id": "be654857-0bc8-40f4-9bc7-8c29f0c575b1",
        "name": "SubmitToTarget",
        "arguments": "{\"job_name\":\"QubitInitialization\",\"target_id\":\"simulator\",\"number_of_shots\":\"1000\"}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"submit_result\":\"Target not available.\"}",
    "toolCallId": "be654857-0bc8-40f4-9bc7-8c29f0c575b1"
  },
  {
    "role": "assistant",
    "content": "It seems the target \"simulator\" is not available. Let's check the available targets for your workspace.\n\n"
  }
]
```
