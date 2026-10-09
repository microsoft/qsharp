# QDK compiler review kit

Reusable instructions and executable helpers for reviewing groups of QDK commits
for panics, crashes, miscompiles, logical errors, and untagged fixes. This is a
review kit, not an automatic correctness proof or a substitute for reading code.

## Supported environment

Linux; Git and the revision's Rust toolchain; a repository-local `.venv` containing
Python 3.10+, the QDK Python dependencies, and the `qirrunner` package's `qir-runner`
executable. Issue snapshots additionally require authenticated `gh`.

The controller uses only the Python standard library. It does not install tools,
alter the existing QDK installation, post to GitHub, commit changes, or modify
production sources. Workers use matching native/Python overlays in disposable
processes. This is crash isolation, **not a sandbox**: execute only inspected,
trusted adapters and Q# programs.

## Start a review

1. Copy `review.example.json` to `review.json` in this directory. Set `reference`
   and `candidate`. Remove optional selectors/exclusions you do not want.
2. Ask your reviewing assistant:

   > Read `review-kit/reviewer.md` and conduct the review configured in
   > `review-kit/review.json`. Use the kit's helpers and report template.

3. The reviewer validates the environment and scope, discovers and curates cases,
   runs the differential matrix, investigates findings, and writes the report.

A skill/custom-agent wrapper is optional. The kit works as ordinary files/scripts.
Do not add this entire procedure to global instructions for unrelated coding work.

## Configuration

Only two fields are required:

```json
{
  "reference": "upstream/main",
  "candidate": "HEAD"
}
```

The configuration's containing Git checkout is the repository. All supplied paths
are relative to the configuration file. `repository` is an optional escape hatch
for configurations stored outside that checkout. The default runtime is
`<repository>/.venv`; the default work directory is `review-kit/.work`.

| Optional field                   | Meaning/default                                                                                 |
| -------------------------------- | ----------------------------------------------------------------------------------------------- |
| `previously_reviewed`            | Previous review's candidate, used to assess new work/rebases; not the regression baseline       |
| `historical_reference`           | Additional explicitly labeled baseline                                                          |
| `issues`                         | Author/minimum-number sweep plus explicit issue includes; both open and closed issues           |
| `clients`                        | Named local Git checkouts to investigate; no chemistry checkout is assumed                      |
| `exclusions`                     | Case ID/origin/path globs, each with a reason                                                   |
| `report`                         | Output path; defaults to `review.md` beside the config                                          |
| `profiles`                       | Base, Adaptive_RI, Adaptive_RIF, Adaptive                                                       |
| `debug_validation`               | `targeted` by default; `all` runs both modes, `none` opts out                                   |
| `keep_qir`                       | `false`: retain failure/uncertain QIR, discard uncomplicated successful QIR; `true` retains all |
| `limits`                         | Stage timeouts, memory, concurrency, shots, seed                                                |
| `venv`, `work_dir`, `repository` | Optional directory overrides; no individual tool-path settings                                  |

Defaults: two concurrent cases; compile 300 seconds; simulator and runner 120
seconds each; 16 GiB address-space limit per worker; eight shots; seed 1.
Timeouts are evidence of incomplete execution, not proof of a compiler bug.
Memory limits also affect subprocesses spawned by the worker.

`review.schema.json` is suitable for editor validation. The CLI additionally
rejects unknown/duplicate fields. It resolves branch names once per command;
subsequent builds within that command use full SHAs. For a multi-command review,
**replace moving refs with the SHAs printed by `check`** before proceeding.

`check` records dirty main/client checkouts. Compiler builds exclude main-worktree
edits. Client cases snapshot local project files, including deliberate edits:
review their dirty state, or use a clean client worktree.

## Commands

Run from the repository root, using its `.venv`:

```bash
.venv/bin/python review-kit/review.py check review-kit/review.json
.venv/bin/python review-kit/review.py build review-kit/review.json
.venv/bin/python review-kit/review.py run review-kit/review.json review-kit/examples/cases.json
```

`run` prints its new run directory. It builds missing revisions automatically.
Results never resume from an old JSONL file: every run is fresh. A run contains
resolved configuration/tools/build identities, case snapshots, stage results,
raw output, and retained unmodified QIR. The main checkout is never switched.

Release is the primary mode. With `debug_validation=targeted`, the reviewer must
assemble a findings manifest and explicitly replay it:

```bash
.venv/bin/python review-kit/review.py run review-kit/review.json review-kit/.work/findings.json --mode debug
```

`debug_validation=all` runs release and debug for every selected case.

Generate an **evidence draft** from a run, then complete it using the report
template. The generated draft does not establish root causes or commit attribution.

```bash
.venv/bin/python review-kit/review.py report review-kit/.work/runs/RUN_DIRECTORY
```

Existing reports are protected; `--overwrite` is only for an owned generated draft.
Helper outputs explicitly named with `--output` can be replaced; choose owned
artifact paths, never an unrelated existing file.

### Discovery

```bash
.venv/bin/python review-kit/review.py samples review-kit/review.json \
  --output review-kit/.work/sample-candidates.json
.venv/bin/python review-kit/review.py issues review-kit/review.json \
  --output review-kit/.work/issues.json
```

Sample discovery uses the pinned candidate checkout (`--revision reference` selects
the other endpoint). It finds Q# files/projects and conservatively infers
zero-argument entries. Inspect the candidates before execution. Ambiguous/skipped
entries appear in the adjacent `.discovery.json`, not as successes. Entries using
implicit file namespaces may need correction. Notebook/OpenQASM/Python scenarios
are selected manually.

Issue discovery snapshots issue bodies and paginated comments, never evaluates or
executes Markdown. Author sweeps include closed issues. Explicit includes override
author/number filters. Commit issue references are added; PR references are recorded
separately. Related issues mentioned only in PR discussions require reviewer
discovery. No author selector means no whole-repository issue sweep.

### Attribution and reduction

```bash
.venv/bin/python review-kit/review.py attribute review-kit/review.json \
  review-kit/.work/findings.json --case CASE_ID \
  --output review-kit/.work/attribution.json
.venv/bin/python review-kit/review.py reduce review-kit/review.json \
  review-kit/.work/findings.json --case CASE_ID --profile Adaptive \
  --kind panic --needle 'failure signature' \
  --output review-kit/.work/reduced.json
```

Attribution evaluates every commit in the range, not a presumed-monotonic binary
search. Results include parents. Neighboring topologically ordered rows are not
necessarily parent/child: confirm the responsible transition on actual parents.
For rebased/non-ancestral comparisons use an explicit ordered `--refs REF ...`.
Failures abort with partial progress saved; an unbuilt commit is not a good commit.

Reduction deletes lines while preserving a specified candidate failure and, by
default, a semantically validated reference. `--allow-preexisting` removes that
reference requirement. Panic/rejection reduction requires a signature; wrong-result
reduction requires a deterministic oracle. It is deliberately simple, bounded by
`--attempts` (default 100), and records exhaustion. It does not establish that the
root cause survived minimization. Projects/adapters must first be reduced manually
to standalone Q#.

## Case manifests

See `cases.schema.json` and `examples/cases.json`. Every case needs a unique ID and
an explicit oracle. Paths are relative to the case manifest.

```json
[
  {
    "id": "measure-one",
    "source_file": "repro.qs",
    "entry": "Main()",
    "oracle": {
      "kind": "deterministic",
      "expected": { "type": "result", "value": 1 }
    },
    "issues": [1234]
  }
]
```

Cases can supply inline `source`, `source_file`, `project_root`, or a trusted
self-contained Python `adapter`; project/source/adapter may be combined where
appropriate. Without an adapter, `entry` is required. An adapter defines
`prepare(ctx) -> (entry_or_callable, args)` and is invoked independently for each
context. Build all Q# callables/UDTs in that context; do not return objects from
another context. Keep local helper code in the adapter itself. Installed client
dependencies must be present in `.venv`; missing dependencies are infrastructure
failures, not compiler diagnostics.

Project snapshots preserve separate local dependency packages. Supported project
layout is `qsharp.json` plus `src/`, with local dependencies declared using `path`.
Source symlinks are rejected explicitly. Other layouts need an adapter/manual
bundle. GitHub dependencies remain remote; pin them to immutable revisions and
document them. QDK GitHub library dependencies use the build's own checkout through
`QDK_LIB_OVERRIDE_MICROSOFT_QDK`; that intentional override is recorded in each build.
Do not describe a manually filtered client project as full-project compatibility.

Oracle policies:

- `deterministic`: the reviewer asserts deterministic _observable return values_.
  An optional hand-derived typed `expected` is strongest. Otherwise both endpoint
  simulators must agree; a sampled singleton is not how determinism is established.
- `observe`: collect complete shots and frequencies without declaring equivalence
  or a miscompile. Use statistical/state checks manually where required.
- `compile_only`: capability/compilation coverage, not a correctness oracle.

Typed values distinguish `result`, `bool`, `int`, `double`, `array`, `tuple`, and
`unit`. Doubles use Python's canonical `float.hex()` spelling (including signed
zero); NaN payload distinctions are not represented. Pure `Unit` return values
never establish semantic equivalence. The kit does not compare messages, quantum
states, or side effects as an oracle.

## Build integrity and storage

Linux builds use `cargo build --locked -p qdk --lib`, with `--release` where selected,
and the configured `.venv` interpreter as `PYO3_PYTHON`. Each overlay combines that
revision's Python package with its resulting `libqdk.so`. This fast path targets the
current QDK layout/ABI; unsupported historical layouts fail explicitly.

Build cache reuse verifies tool/dependency metadata, package/library digests,
worktree SHA and non-ignored modifications; workers verify actual import locations and
the native digest. Compilation failures stop the build, never reuse a stale library.
No result cache or silent QIR patch is used.

`.work` is ignored and disposable, but includes Git-registered worktrees. Do not
recursively delete it while those registrations remain. After inspecting ownership,
remove only specific kit worktrees with `git worktree remove PATH`, then their
specific build/run artifacts. Never remove another review's scratch/worktrees.

Keep configuration examples, prompts, small curated repros, generators/seeds, and
compact final evidence. Do not check in native binaries, bulk generated cases, or
all QIR. Cold builds need dependencies and disk space, but no saved corpus.

## Validate the kit

```bash
.venv/bin/python -m unittest discover -s review-kit/tests -v
.venv/bin/python review-kit/tests/smoke.py
```

The smoke config compares `HEAD^` with `HEAD`; this tests plumbing, not arbitrary
branch correctness. Override those refs for revisions you intend to validate.
Fixtures cover all profiles, typed/nested outputs, signed zero, stochastic
observations, callable arguments, and separate local project dependencies.
`tests/smoke.py --mode debug` exercises debug builds. It asserts the expected
profile/status matrix and exact shot counts, rather than accepting exit zero alone.
No chemistry-specific trace interpreter/state checker is included.
