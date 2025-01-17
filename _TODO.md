# TODO

- Get a table response for 'last 5 jobs tagged carbon'
- Have a button that fetches the histogram for the job
- Add an embedding for a 'histogram' widget and see if it renders

# Punt until next release:

- Take semantic kernel out of the business of invoking tool calls.
- Figure out integration with vscode copilot.

# Test scripts

## Environments:

- No workspaces added.
- One workspace
- More than one workspace

## Scenarios:

- user: "show me my jobs"
  - assistant: Should pick up on the fact that there are no workspaces, and should prompt for adding one.

## Unsuccessful conversation 1:

Problems:

- "Let's retrieve your recent jobs" should have been a tool call
- "anyway can you submit this program to azure quantum?\n" should have provoked a tool call

```json
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

## Unsuccessful conversation 2:

This was with OpenAI. Azure Quantum actually doesn't seem to have this problem.

Problems:

- Model is struggling because it has no concept of the current program.

```json
[
  {
    "role": "user",
    "content": "submit this to azure pls\n"
  },
  {
    "role": "assistant",
    "content": "Sure, I can help with that. Please provide the target information and any parameters you would like to include for the submission."
  },
  {
    "role": "user",
    "content": "what are my options?\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "id": "7799e41c-34cf-4fa5-ad2e-be9ee64bfca0",
        "name": "GetProviders",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"providers\":[]}",
    "toolCallId": "7799e41c-34cf-4fa5-ad2e-be9ee64bfca0"
  },
  {
    "role": "assistant",
    "content": "It looks like there are no hardware providers available in your current workspace. Please ensure you have access to an Azure Quantum Workspace with available providers. You can also connect to a different workspace if needed."
  },
  {
    "role": "user",
    "content": "err can you check again pls\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "id": "b13e038b-b2a0-47b5-b063-f1931fc176ff",
        "name": "GetProviders",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"providers\":[{\"providerId\":\"ionq\",\"currentAvailability\":\"Available\",\"targets\":[{\"id\":\"ionq.qpu.aria-1\",\"currentAvailability\":\"Unavailable\",\"averageQueueTime\":0,\"statusPage\":\"https://status.ionq.co\"},{\"id\":\"ionq.qpu.aria-2\",\"currentAvailability\":\"Unavailable\",\"averageQueueTime\":0,\"statusPage\":\"https://status.ionq.co\"},{\"id\":\"ionq.simulator\",\"currentAvailability\":\"Available\",\"averageQueueTime\":3,\"statusPage\":\"https://status.ionq.co\"}]},{\"providerId\":\"quantinuum\",\"currentAvailability\":\"Available\",\"targets\":[{\"id\":\"quantinuum.qpu.h1-1\",\"currentAvailability\":\"Degraded\",\"averageQueueTime\":0,\"statusPage\":\"https://www.quantinuum.com/hardware/h1\"},{\"id\":\"quantinuum.sim.h1-1sc\",\"currentAvailability\":\"Available\",\"averageQueueTime\":0,\"statusPage\":\"https://www.quantinuum.com/hardware/h1\"},{\"id\":\"quantinuum.sim.h1-1e\",\"currentAvailability\":\"Available\",\"averageQueueTime\":26749,\"statusPage\":\"https://www.quantinuum.com/hardware/h1\"}]},{\"providerId\":\"rigetti\",\"currentAvailability\":\"Available\",\"targets\":[{\"id\":\"rigetti.sim.qvm\",\"currentAvailability\":\"Available\",\"averageQueueTime\":5,\"statusPage\":\"https://rigetti.statuspage.io/\"},{\"id\":\"rigetti.qpu.ankaa-9q-3\",\"currentAvailability\":\"Available\",\"averageQueueTime\":5,\"statusPage\":\"https://rigetti.statuspage.io/\"},{\"id\":\"rigetti.qpu.ankaa-3\",\"currentAvailability\":\"Available\",\"averageQueueTime\":5,\"statusPage\":\"https://rigetti.statuspage.io/\"}]}]}",
    "toolCallId": "b13e038b-b2a0-47b5-b063-f1931fc176ff"
  },
  {
    "role": "assistant",
    "content": "Here are your available options for submission:\n\n### IonQ\n- **ionq.qpu.aria-1** (Unavailable)\n- **ionq.qpu.aria-2** (Unavailable)\n- **ionq.simulator** (Available, Avg. Queue Time: 3 mins)\n\n### Quantinuum\n- **quantinuum.qpu.h1-1** (Degraded)\n- **quantinuum.sim.h1-1sc** (Available)\n- **quantinuum.sim.h1-1e** (Available, Avg. Queue Time: 26749 mins)\n\n### Rigetti\n- **rigetti.sim.qvm** (Available, Avg. Queue Time: 5 mins)\n- **rigetti.qpu.ankaa-9q-3** (Available, Avg. Queue Time: 5 mins)\n- **rigetti.qpu.ankaa-3** (Available, Avg. Queue Time: 5 mins)\n\nPlease specify the target you would like to submit your job to."
  },
  {
    "role": "user",
    "content": "ok, ionq.simulator\n"
  },
  {
    "role": "assistant",
    "content": "Please provide a name for your job and the number of shots you would like to use for the submission."
  },
  {
    "role": "user",
    "content": "\"foo\", 1 shot\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "arguments": "{\"job_name\":\"foo\",\"target_id\":\"ionq.simulator\",\"number_of_shots\":1}",
        "id": "call_ygud7a8omUwE4fjWSvE9VnRG",
        "name": "SubmitToTarget"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"submit_result\":\"Error: QIR generation failed due to error: \\\"[{\\\"document\\\":\\\"file:///c%3A/src/qsharp/samples/estimation/EkeraHastadFactoring.qs\\\",\\\"diagnostic\\\":{\\\"range\\\":{\\\"start\\\":{\\\"line\\\":27,\\\"character\\\":4},\\\"end\\\":{\\\"line\\\":27,\\\"character\\\":127}},\\\"message\\\":\\\"cannot call a cyclic operation\\\\n\\\\nhelp: calling an operation specialization that contains call cycles is not supported by the configured target profile\\\",\\\"severity\\\":\\\"error\\\",\\\"code\\\":\\\"Qsc.CapabilitiesCk.CallToCyclicOperation\\\",\\\"uri\\\":\\\"https://aka.ms/qdk.qir#call-to-cyclic-operation\\\"},\\\"stack\\\":null},{\\\"document\\\":\\\"file:///c%3A/src/qsharp/samples/estimation/EkeraHastadFactoring.qs\\\",\\\"diagnostic\\\":{\\\"range\\\":{\\\"start\\\":{\\\"line\\\":59,\\\"character\\\":4},\\\"end\\\":{\\\"line\\\":59,\\\"character\\\":46}},\\\"message\\\":\\\"cannot call a cyclic operation\\\\n\\\\nhelp: calling an operation specialization that contains call cycles is not supported by the configured target profile\\\",\\\"severity\\\":\\\"error\\\",\\\"code\\\":\\\"Qsc.CapabilitiesCk.CallToCyclicOperation\\\",\\\"uri\\\":\\\"https://aka.ms/qdk.qir#call-to-cyclic-operation\\\"},\\\"stack\\\":null},{\\\"document\\\":\\\"file:///c%3A/src/qsharp/samples/estimation/EkeraHastadFactoring.qs\\\",\\\"diagnostic\\\":{\\\"range\\\":{\\\"start\\\":{\\\"line\\\":60,\\\"character\\\":4},\\\"end\\\":{\\\"line\\\":60,\\\"character\\\":49}},\\\"message\\\":\\\"cannot call a cyclic operation\\\\n\\\\nhelp: calling an operation specialization that contains call cycles is not supported by the configured target profile\\\",\\\"severity\\\":\\\"error\\\",\\\"code\\\":\\\"Qsc.CapabilitiesCk.CallToCyclicOperation\\\",\\\"uri\\\":\\\"https://aka.ms/qdk.qir#call-to-cyclic-operation\\\"},\\\"stack\\\":null},{\\\"document\\\":\\\"file:///c%3A/src/qsharp/samples/estimation/EkeraHastadFactoring.qs\\\",\\\"diagnostic\\\":{\\\"range\\\":{\\\"start\\\":{\\\"line\\\":136,\\\"character\\\":4},\\\"end\\\":{\\\"line\\\":136,\\\"character\\\":47}},\\\"message\\\":\\\"cannot call a cyclic operation\\\\n\\\\nhelp: calling an operation specialization that contains call cycles is not supported by the configured target profile\\\",\\\"severity\\\":\\\"error\\\",\\\"code\\\":\\\"Qsc.CapabilitiesCk.CallToCyclicOperation\\\",\\\"uri\\\":\\\"https://aka.ms/qdk.qir#call-to-cyclic-operation\\\"},\\\"stack\\\":null},{\\\"document\\\":\\\"file:///c%3A/src/qsharp/samples/estimation/EkeraHastadFactoring.qs\\\",\\\"diagnostic\\\":{\\\"range\\\":{\\\"start\\\":{\\\"line\\\":137,\\\"character\\\":4},\\\"end\\\":{\\\"line\\\":137,\\\"character\\\":66}},\\\"message\\\":\\\"cannot call a cyclic operation\\\\n\\\\nhelp: calling an operation specialization that contains call cycles is not supported by the configured target profile\\\",\\\"severity\\\":\\\"error\\\",\\\"code\\\":\\\"Qsc.CapabilitiesCk.CallToCyclicOperation\\\",\\\"uri\\\":\\\"https://aka.ms/qdk.qir#call-to-cyclic-operation\\\"},\\\"stack\\\":null}]\\\". Please ensure the code is compatible with a QIR profile by setting the target QIR profile to 'base' or 'adaptive_ri' and fixing any errors.\"}",
    "toolCallId": "call_ygud7a8omUwE4fjWSvE9VnRG"
  },
  {
    "role": "assistant",
    "content": "The submission encountered an error related to cyclic operations in your Q# program. Specifically, it states:\n\n- \"cannot call a cyclic operation\" \n- The configured target profile does not support calling operations with cycles.\n\nTo resolve this issue, you may need to:\n1. Ensure that the operations in your Q# code do not have cycles.\n2. Adjust the target QIR profile to 'base' or 'adaptive_ri' if applicable.\n\nPlease review your code and make the necessary changes, then try to submit again. Let me know if you need further assistance!"
  },
  {
    "role": "user",
    "content": "can you submit this current program instead?\n"
  },
  {
    "role": "assistant",
    "content": "I don't have access to the current program you are referring to. Please provide the program code or specify any adjustments you'd like to make so I can assist you in submitting it."
  },
  {
    "role": "user",
    "content": "this one right here that's open in the editor\n"
  },
  {
    "role": "assistant",
    "content": "I'm unable to directly access or retrieve the code from your editor. Please copy and paste the Q# program code here, and I'll assist you in submitting it to Azure."
  },
  {
    "role": "user",
    "content": "yes you can\n"
  },
  {
    "role": "assistant",
    "content": "I apologize for any misunderstanding, but I don’t have the ability to access the current program in your editor. Please provide the code, and I’d be happy to assist you in submitting it!"
  },
  {
    "role": "user",
    "content": "just try it!\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "arguments": "{\"job_name\":\"foo\",\"target_id\":\"ionq.simulator\",\"number_of_shots\":1}",
        "id": "call_45rsqbh95PyZCxfs3TNoUkz4",
        "name": "SubmitToTarget"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"submit_result\":\"Job submitted successfully with ID: 419f2b4c-be71-408c-8cf0-2710fa5dabee\"}",
    "toolCallId": "call_45rsqbh95PyZCxfs3TNoUkz4"
  },
  {
    "role": "assistant",
    "content": "The job has been submitted successfully with the ID: **419f2b4c-be71-408c-8cf0-2710fa5dabee**. \n\nIf you need further assistance, feel free to ask!"
  }
]
```

## Unsuccessful conversation 3:

Problems:

- Hallucinates target information.

```json
[
  {
    "role": "user",
    "content": "submit this to azure pls\n"
  },
  {
    "role": "assistant",
    "content": "Please provide the target information and any necessary parameters for submission. For example:\n\n- Target (e.g., `ionq.simulator`, `quantinuum.hqs-lt-s1-apival`)\n- Shots (number of times to run the program)\n- Any other relevant parameters\n\nOnce I have this information, I can proceed with the submission."
  },
  {
    "role": "user",
    "content": "what are my options?\n"
  },
  {
    "role": "assistant",
    "content": "",
    "ToolCalls": [
      {
        "id": "95ae0cc6-79f5-4d9d-976c-66b3e1ecf78d",
        "name": "GetProviders",
        "arguments": "{}"
      }
    ]
  },
  {
    "role": "tool",
    "content": "{\"providers\":[{\"providerId\":\"ionq\",\"currentAvailability\":\"Available\",\"targets\":[{\"id\":\"ionq.qpu.aria-1\",\"currentAvailability\":\"Unavailable\",\"averageQueueTime\":0,\"statusPage\":\"https://status.ionq.co\"},{\"id\":\"ionq.qpu.aria-2\",\"currentAvailability\":\"Unavailable\",\"averageQueueTime\":0,\"statusPage\":\"https://status.ionq.co\"},{\"id\":\"ionq.simulator\",\"currentAvailability\":\"Available\",\"averageQueueTime\":1,\"statusPage\":\"https://status.ionq.co\"}]},{\"providerId\":\"quantinuum\",\"currentAvailability\":\"Available\",\"targets\":[{\"id\":\"quantinuum.qpu.h1-1\",\"currentAvailability\":\"Available\",\"averageQueueTime\":4935351,\"statusPage\":\"https://www.quantinuum.com/hardware/h1\"},{\"id\":\"quantinuum.sim.h1-1sc\",\"currentAvailability\":\"Available\",\"averageQueueTime\":0,\"statusPage\":\"https://www.quantinuum.com/hardware/h1\"},{\"id\":\"quantinuum.sim.h1-1e\",\"currentAvailability\":\"Available\",\"averageQueueTime\":27094,\"statusPage\":\"https://www.quantinuum.com/hardware/h1\"}]},{\"providerId\":\"rigetti\",\"currentAvailability\":\"Available\",\"targets\":[{\"id\":\"rigetti.sim.qvm\",\"currentAvailability\":\"Available\",\"averageQueueTime\":5,\"statusPage\":\"https://rigetti.statuspage.io/\"},{\"id\":\"rigetti.qpu.ankaa-9q-3\",\"currentAvailability\":\"Available\",\"averageQueueTime\":5,\"statusPage\":\"https://rigetti.statuspage.io/\"},{\"id\":\"rigetti.qpu.ankaa-3\",\"currentAvailability\":\"Available\",\"averageQueueTime\":5,\"statusPage\":\"https://rigetti.statuspage.io/\"}]}]}",
    "toolCallId": "95ae0cc6-79f5-4d9d-976c-66b3e1ecf78d"
  },
  {
    "role": "assistant",
    "content": "Here are your available targets:\n\n### IonQ\n- `ionq.qpu.aria-1` (Unavailable)\n- `ionq.qpu.aria-2` (Unavailable)\n- `ionq.simulator` (Available)\n\n### Quantinuum\n- `quantinuum.qpu.h1-1` (Available, average queue time: 4935351 seconds)\n- `quantinuum.sim.h1-1sc` (Available)\n- `quantinuum.sim.h1-1e` (Available, average queue time: 27094 seconds)\n\n### Rigetti\n- `rigetti.sim.qvm` (Available, average queue time: 5 seconds)\n- `rigetti.qpu.ankaa-9q-3` (Available, average queue time: 5 seconds)\n- `rigetti.qpu.ankaa-3` (Available, average queue time: 5 seconds)\n\nPlease specify the target you would like to submit to and any additional parameters such as the number of shots."
  },
  {
    "role": "user",
    "content": "ionq.simulator\n"
  },
  {
    "role": "assistant",
    "content": "Please provide the number of shots (the number of times to run the program) and any other relevant parameters for the submission."
  }
]
```
