---
name: Fuzzing Bug Report
about: Used by fuzzing to automatically file the bugs found.
title: 'Fuzz: "{{ env.PANIC_MESSAGE }}" ({{ env.WF_AGENT_OS }})'
labels: bug
assignees: kuzminrobin
---

The fuzz-testing workflow has detected a bug.

<details><summary><b>Auto-Minimized Fuzzing Input That Triggers the Bug:</b> Click this line.</summary>
<b>Note:</b> If the input is multi-line then the end-of-line characters '\n' (0x0A) and '\r' (0x0D)
may affect the reproducibility of the bug. If you fail to repro the bug with the input shown below
then you may want to go to the <a href={{ env.WORKFLOW_RUN_URL }}>workflow</a> that reported this GitHub bug,
download the artifact, and extract the file with the exact minimized input.

```qs
{{ env.MINIMIZED_INPUT }}
```

</details>

<details><summary><b>Fuzzing <code>stderr</code> Log</b> (last 62kB), includes the stack trace: Click.</summary>
The fragment of interest starts with "panicked at".

```gdb
{{ env.FUZZ_STDERR_LOG }}
```

</details>

<details><summary><b>The commit in <code>main</code> the bug has been found in:</b> Click.</summary>
If the developers fail to repro the bug in the latest <code>main</code> then the commit info below can help them to make sure
that they are using the correct way to repro. If the bug is reproducible in the commit below, but not in latest <code>main</code>,
then the bug is likely fixed already.

```log
{{ env.COMMIT_INFO }}
```

</details>

**Other Info**

- [**Workflow**]({{ env.WORKFLOW_RUN_URL }}) (contains the run artifacts).
- **Workflow Agent System Info:** `{{ env.WF_AGENT_OS }}: {{ env.WF_AGENT_SYS_INFO }}`.
- **Bug Reporting Timestamp:** {{ date | date('YYYY.MM.DD HH:mm UTC') }} (UTC - 8:00 = PST. UTC - 7:00 = PDT).
