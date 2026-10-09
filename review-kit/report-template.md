# Compiler review: `<candidate>` versus `<reference>`

## Summary

Reference: `<full SHA>`  
Candidate: `<full SHA>`  
Historical baseline: `<full SHA or none>`  
Previously reviewed / rebase mapping: `<details or none>`

| Finding      | Classification                      | Profiles     | Introducing/fixing commit | Related issue              | Client impact           |
| ------------ | ----------------------------------- | ------------ | ------------------------- | -------------------------- | ----------------------- |
| `<ID/title>` | `<regression/fixed/non-regression>` | `<profiles>` | `<SHA or unestablished>`  | `<issue/new trigger/none>` | `<client and evidence>` |

## 1. Regressions

### R1. `<specific behavior>`

**Expected / observed:** `<precise results or failure>`  
**Affected profiles/builds:** `<matrix>`  
**Introduced by:** `<commit and parent evidence, or incomplete attribution>`  
**Related issues:** `<links and relationship>`  
**Client impact:** `<demonstrated effect or explicitly untested>`  
**Cause/confidence:** `<mechanism, locations, uncertainty>`

```qsharp
// Replace with a minimal runnable program.
```

```python
# Replace with a standalone executed repro using .venv's selected QDK build.
# Derive qir-runner from the interpreter directory; retain raw exit/error output.
```

| Revision | Mode              | Profile     | Simulator    | Compilation  | Runner       | Outcome            |
| -------- | ----------------- | ----------- | ------------ | ------------ | ------------ | ------------------ |
| `<SHA>`  | `<release/debug>` | `<profile>` | `<observed>` | `<observed>` | `<observed>` | `<interpretation>` |

Observed output:

```text
<actual output, including failures and exit statuses>
```

Repeat this full structure for every unresolved issue. Do not substitute a local
absolute project path for a self-contained repro.

## 2. Fixed issues

For each: original repro, reference failure, candidate's validated correct
behavior, profiles/modes, fixing commit, tested variants, remaining limitations,
and whether commits/PRs reference the issue. Separate partial fixes and acceptance
without semantic validation.

## 3. Non-regression issues

Use the full unresolved-issue structure above. Show the reference failure; explain
whether this is the original trigger, a new trigger, or a demonstrated distinct
root cause. Include configured-client impact.

## Issue status table

| Issue      | State at snapshot | Tested repro/variant | Profiles     | Reference  | Candidate  | Assessment                                                                      |
| ---------- | ----------------- | -------------------- | ------------ | ---------- | ---------- | ------------------------------------------------------------------------------- |
| `<number>` | `<open/closed>`   | `<case>`             | `<profiles>` | `<result>` | `<result>` | `<repro/fixed/partial/newly accepted/unsupported/no runnable repro/not tested>` |

## Coverage and limitations

- Run manifests, exact tool/dependency versions, build/import provenance.
- Native suites and end-to-end cases, source/API surfaces and profiles.
- Generated families, seeds, budgets and reduction results.
- Release/debug coverage; stochastic/Unit/state checks and oracle limitations.
- Full client projects versus explicitly named subsets; client revisions/edits.
- Exclusions with reasons, pending discovery, unsupported profiles, failed builds,
  infrastructure errors, timeouts, and incomplete attribution.
- Final inline repro execution evidence.

Do not interpret “no observed regression” as exhaustive correctness.
