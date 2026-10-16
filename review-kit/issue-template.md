# Issue drafting template

Use this when the user asks for issues to file. Draft only; never file issues or post
comments unless separately authorized. Write one sub-issue per distinct trigger (a
shared panic site is not enough to merge two), separated by `---`, plus a prose
summary for a meta-issue. Prefer Python repros that call the `qdk` package directly
so maintainers can paste and run them; keep Q# inside the script.

## Meta-issue summary

A few paragraphs, no tables required:

- What was reviewed (PRs/commits, `main` SHA) and how the issues were found (suites,
  generators, client scenarios), including the client usage patterns that matter.
- Groups of issues with a likely shared cause, and the evidence for it (for example
  a common field in the panic messages).
- Which issues are regressions from the change under review and which also reproduce
  on `main`.
- Related issues already tracked elsewhere, and their status.

---

## [Panic] <crate>: <short description of the trigger>

- **Crate:** `<crate>`
- **Panic:** [<file>:<line>](../../source/<path>#L<line>), in `<function>` (from `review.py exec`; say which revision the line refers to when it differs from `main`)
- **Stage / profile:** QIR compile (`context.compile`), <profiles>; <variants that do or do not reproduce>
- **Message:** `<first line of the panic message>`
- **Revisions:** `main` (`<sha>`): <behavior>; <candidate> (`<sha>`): <behavior>. Mark regressions explicitly.
- **Related:** #<n> (<how it overlaps>), ...

```python
from qdk import Context, TargetProfile

SRC = """
// minimal Q#
"""

ctx = Context(target_profile=TargetProfile.Adaptive_RIF)
ctx.eval(SRC)
print(ctx.compile("Main()"))  # expected: <hand-derived result>
```

---

For miscompiles use `[Miscompile]` in the title, replace **Panic** with the
observed versus expected values, and state how the expectation was derived. For
client-dependent repros, name the client repository and commit, and keep every
other input user-defined.
