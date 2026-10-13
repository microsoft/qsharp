# qdk_cutensornet

## What this crate does

`qdk_cutensornet` is the Rust boundary between the QDK and NVIDIA cuTensorNet.
It finds and loads cuTensorNet and the CUDA Runtime at run time, so optional GPU
acceleration does not make CUDA part of the QDK's ordinary build or startup
contract. On top of that loader it runs the MPS method (block ④ of the
[tensor-network design](../TensorNetworkDesign.md)): it simulates a program's
state as a _matrix product state_ (MPS), a chain of site tensors whose bonds
are truncated to at most χ, on the GPU. See [The MPS method](#the-mps-method).

Native shared libraries expose functions under exported names called
_symbols_, such as `cudaMalloc` or `cutensornetCreate`. To keep NVIDIA software
optional, QDK does not link those functions into every build. When `discover()`
is called, this crate opens compatible libraries installed on the host, looks
up the approved symbols by name, and stores their addresses as typed Rust
function pointers. This lookup is called _symbol resolution_; resolving a
function does not execute it or perform GPU work.

The crate maintains two approved function surfaces:

- The CUDA Runtime functions declared in `src/bindings/cudart_12.rs` and
  assembled into the handwritten `CudaFunctions` table in `src/library.rs`.
  This surface currently contains 12 required functions. The `_12` suffix
  identifies CUDA Runtime major version 12; it is not the function count.
- The cuTensorNet functions listed in `scripts/cutensornet-symbols.txt`. This
  surface currently contains 54 required functions and the optional
  `cutensornetGetLastError`; its typed loader tables are generated from the
  audited declarations in `src/bindings/v2_13.rs`.

The handwritten ABI files serve different purposes:
`src/bindings/cudart_12.rs` contains hand-audited CUDA Runtime declarations,
while `src/bindings/mod.rs` contains hand-audited compile-time assertions that
verify selected layout and constant assumptions across both binding surfaces.

`discover()` opens both libraries, resolves these function surfaces, validates
their versions, and returns an `Availability` value that keeps the libraries
loaded. Code executing a later cuTensorNet-backed workload uses the stored
function pointers; discovery itself performs no GPU work.

The loader explicitly discovers and validates both libraries. It is Linux
x86-64 only and introduces no native link-time dependency; instead, it uses
[explicit runtime dynamic loading](#why-dynamic-loading). Building the QDK,
loading another QDK component, and using CPU simulation require none of the
following:

- an NVIDIA GPU;
- a CUDA or cuTensorNet installation;
- NVIDIA headers or shared libraries; or
- a CUDA-aware linker configuration.

On other platforms the crate compiles, `discover()` reports the libraries as
unavailable, and the MPS entry points report that as an environment error.

## Lifecycle at a glance

Most users and contributors do not run code generation. The cuTensorNet Rust
declarations and dynamic-loader tables are generated ahead of time from a
pinned NVIDIA SDK, reviewed, and committed to this repository. Ordinary Cargo
builds compile those checked-in files and require neither the SDK archive nor
NVIDIA headers.

Maintainers rerun code generation only when they need to update or reproduce
those files; see the [FFI generation documentation](scripts/README.md).
Separately, at run time, a QDK component calls `discover()` only when preparing
a cuTensorNet-backed workload. That operation looks for compatible cuTensorNet
and CUDA libraries installed on the host. Each call into the MPS method checks
the program on the host first, then discovers the libraries, opens one session
on the device, computes one state, answers the call from it and closes the
session.

```text
pinned NVIDIA headers
  -> maintainer generation
  -> reviewed, checked-in Rust source
  -> ordinary Cargo build
  -> per MPS call: host checks
       -> optional runtime library discovery
       -> one session, one state
       -> results, session closed
```

## What this crate contains

The implementation is split into narrow layers so that generated ABI facts,
runtime availability and native resources do not acquire overlapping
responsibilities.

The crate works with two native shared libraries. The CUDA Runtime
(`libcudart.so.12`) provides the selected CUDA version, device, memory, and
stream APIs. NVIDIA cuTensorNet (`libcutensornet.so.2`) provides the tensor
network API. Discovery loads and validates both libraries, although it invokes
only their approved version and error probes and performs no GPU work.

| Layer                     | Location                                         | Responsibility                                                                                                                 |
| ------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| Generated cuTensorNet ABI | `src/bindings/v2_13.rs`                          | Reduced declarations generated from the audited cuTensorNet 2.13 header.                                                       |
| Audited CUDA Runtime ABI  | `src/bindings/cudart_12.rs`                      | Hand-audited declarations for the approved CUDA Runtime functions; `_12` denotes CUDA major version 12.                        |
| ABI assertions            | `src/bindings/mod.rs`                            | Hand-audited compile-time size, alignment, offset, and selected constant checks.                                               |
| Version policy            | `src/version.rs`                                 | Accepts only the audited cuTensorNet and CUDA Runtime versions.                                                                |
| Dynamic loader            | `src/library.rs`                                 | At run time, opens both libraries, resolves approved exports into typed pointers, probes versions, and retains library guards. |
| Generated loader tables   | `src/library/symbols{,/*}.rs`                    | Generated cuTensorNet function-table fields and per-symbol runtime lookup calls.                                               |
| Loader generator          | `src/generator.rs`, `src/bin/generate-loader.rs` | Maintainer tool that reads the manifest and binding signatures and writes the generated loader tables.                         |
| Availability API          | `src/lib.rs`                                     | Exposes explicit discovery and reports availability without exposing raw FFI.                                                  |
| Error and status model    | `src/error.rs`                                   | Defines structured discovery failures and maps native status values without losing unknown codes.                              |
| MPS entry                 | `src/execution.rs`                               | The MPS method's entry points for the Python API: host checks, one session per call, results.                                  |
| MPS mechanics             | `src/simulation.rs`, `src/library/simulation/`   | cuTensorNet-specific state lifecycle, sampler, region consumer and policy; host-tested through private injection traits.       |
| MPS native adapter        | `src/library/simulation{.rs,/mps_session.rs}`    | `simulation.rs` implements the four traits through the loaded function tables; `mps_session.rs` binds policy to one device.    |

The raw bindings, function tables, library guards, resolver seam, and native
status mapping are private. The public capability is
`discover() -> Result<Availability, AvailabilityError>`. The MPS entry points
are public only so that the Python native module can call them; they are
`#[doc(hidden)]` and not a stable API.

## Runtime discovery

Discovery is explicit and lazy. No global loader cache or startup hook exists.
Code that intends to execute a cuTensorNet-backed workload calls `discover()`
while preparing that workload, handles any `AvailabilityError` by reporting it
or selecting an allowed fallback, and retains the returned `Availability` for
as long as its native function pointers are needed. Code that does not request
cuTensorNet acceleration never triggers discovery.

If `discover()` is never called, this crate does not inspect the host, open a
native library, probe a version, perform GPU work, or produce an error. There is
no hidden operation that requires prior initialization.

In the transaction below, _resolve_ means looking up each exported function by
name and storing its address as the corresponding typed function pointer. It
does not mean calling the function.

The discovery transaction is:

1. Reject targets other than Linux x86-64 with `UnsupportedPlatform`.
2. Validate any explicit library overrides before loading either library.
3. Open the CUDA Runtime with `RTLD_NOW | RTLD_LOCAL`.
4. Resolve the currently approved CUDA Runtime functions declared in
   `src/bindings/cudart_12.rs` into the handwritten `CudaFunctions` table.
5. Open cuTensorNet with `RTLD_NOW | RTLD_LOCAL`.
6. Resolve the required and optional cuTensorNet functions listed in
   `scripts/cutensornet-symbols.txt` into the generated function tables.
7. Probe the loaded CUDA Runtime, CUDA driver API, cuTensorNet runtime, and the
   CUDA Runtime ABI used to build cuTensorNet.
8. Apply the exact version policy and return an `Availability` value that keeps
   both libraries alive longer than every copied function pointer.

Construction is all-or-nothing. Every opened library is immediately protected
by a Rust RAII guard, and no partially initialized function table or
availability value can escape on failure. This is implemented in Rust in
`src/library.rs`: `LoadedLibrary` owns a `libloading::Library`, local values are
advanced through each fallible step with `Result` and `?`, and Rust drops every
successfully constructed guard if a later step returns an error.
`libloading::Symbol` values are not retained; plain typed function pointers are
copied out while their owning libraries stay alive.

### Loader flags

On Linux, `libloading` opens each shared object through `dlopen` with
`RTLD_NOW | RTLD_LOCAL`:

- `RTLD_NOW` requires the operating-system loader to resolve undefined symbols
  before the open succeeds. Missing transitive dependencies therefore fail at
  discovery instead of appearing later during simulation.
- `RTLD_LOCAL` keeps symbols from the opened object out of the process-global
  symbol namespace. This avoids making QDK's optional backend an implicit
  dependency of subsequently loaded components and reduces symbol collisions.

Together, these flags provide an early, bounded validation point without
polluting global process state. Transitive dependencies are still resolved by
the operating-system loader according to its normal rules.

### Failure reporting

Discovery failures preserve the context needed by a provider or user to
diagnose the installation:

- unsupported target platform;
- invalid override variable, path, and reason;
- every default path or loader name attempted for a missing library;
- the selected library path and native loader message for load or transitive
  dependency failures;
- the library path and exact missing required symbol;
- the component, observed version, and accepted version for policy failures;
  and
- the native status and copied CUDA error text for failed version probes.

### Search policy

| Component    | Exclusive override        | Ordered defaults                                                                            |
| ------------ | ------------------------- | ------------------------------------------------------------------------------------------- |
| cuTensorNet  | `QDK_CUTENSORNET_LIBRARY` | `/usr/lib/x86_64-linux-gnu/libcuquantum/12/libcutensornet.so.2`, then `libcutensornet.so.2` |
| CUDA Runtime | `QDK_CUDART_LIBRARY`      | `/usr/local/cuda-12.9/targets/x86_64-linux/lib/libcudart.so.12`, then `libcudart.so.12`     |

An override must be an absolute path to an existing regular file or symlink.
When set, it is attempted exclusively, so a bad deployment configuration does
not silently fall back to another installation. The default search is finite
and ordered; this crate does not walk the filesystem or guess unreviewed
versions. Transitive native dependencies are resolved by the operating-system
loader, and their failures retain the attempted path and loader message.

### Version policy

The accepted values are deliberately exact:

- cuTensorNet runtime: `21300` (2.13.0);
- CUDA Runtime: `12090` (12.9); and
- cuTensorNet's reported CUDA Runtime ABI: `12090`.

The CUDA driver API version is reported for diagnostics. It is not substituted
for either CUDA Runtime check. Unknown versions fail before device selection or
native handle creation; a newer version is not assumed ABI-compatible.

## FFI and ABI boundary

The FFI layer describes how Rust calls the C libraries. The ABI policy defines
the exact binary layouts, calling conventions, constants, symbols, and runtime
versions for which those calls have been audited. Both are intentionally
private and narrower than the complete NVIDIA APIs.

### Scoped declarations

- `src/bindings/v2_13.rs` is generated from the checksum-identified official
  cuTensorNet 2.13 header. It contains the selected functions required for
  discovery, state replay, expectations, sampling, general contraction and
  logging, plus the optional `cutensornetGetLastError` diagnostic and their
  required type/constant closure.
- `src/bindings/cudart_12.rs` contains hand-audited declarations for the
  currently approved CUDA Runtime functions needed by discovery and by device,
  memory and stream management. The `_12` suffix denotes CUDA Runtime major
  version 12, not the number of declared functions.
- Function pointers use exact private `unsafe extern "C"` types. Unsafe loading,
  symbol resolution, and native probe calls remain inside the crate; the public
  `discover()` API is safe Rust.
- The owning `libloading::Library` guards outlive all copied function pointers.
  Borrowed native C strings are copied into Rust-owned `String` values before
  another native call can invalidate them.
- Discovery resolves the full approved tables atomically but invokes only
  version and error probes. Resolving the state, workspace, memory, and
  stream calls does not execute them.

Ordinary Cargo builds consume only the checked-in Rust declarations. They do
not run bindgen, inspect an SDK header, execute a build script, or pass a native
link directive.

### Compile-time ABI checks

On the supported Linux x86-64 target, `src/bindings/mod.rs` rejects compilation
if the selected ABI facts drift. It checks:

- 32-bit size and alignment for represented cuTensorNet/CUDA status and enum
  values;
- 64-bit size and alignment for opaque cuTensorNet handles and CUDA streams;
- a 16-byte size and alignment, with real and imaginary fields at byte offsets
  0 and 8, for the private complex-f64 storage type intended for
  `CUDA_C_64F` buffers; and
- selected constants including `CUDA_C_64F` and host-to-device and
  device-to-host copy directions.

Path and slicing attribute payloads also have size, alignment and field-offset
assertions. Their Linux x86-64 layouts, in bytes, are:

| Type                           | Size | Alignment | Field offsets                      |
| ------------------------------ | ---- | --------- | ---------------------------------- |
| `cutensornetNodePair_t`        | 8    | 4         | `first`: 0, `second`: 4            |
| `cutensornetContractionPath_t` | 16   | 8         | `numContractions`: 0, `data`: 8    |
| `cutensornetSliceInfoPair_t`   | 16   | 8         | `slicedMode`: 0, `slicedExtent`: 8 |
| `cutensornetSlicingConfig_t`   | 16   | 8         | `numSlicedModes`: 0, `data`: 8     |

The selected cuTensorNet calls pass complex tensor storage through device
`void *` buffers rather than passing a complex struct by value. The safe layer
above the bindings must use the asserted private storage representation and explicit
conversion instead of assuming that a general Rust complex type has the C ABI.

These assertions do not claim compatibility with another operating system,
architecture, CUDA major version, header ABI, or cuTensorNet runtime. Support
for any of those requires a separate retained audit. Native status conversion
also preserves unknown numeric values so a new status is not silently
misclassified.

## Why dynamic loading

Runtime loading is the best fit for this optional QDK capability because it:

- keeps ordinary QDK builds and packages independent of proprietary GPU
  installations;
- prevents a missing NVIDIA library from breaking process startup or unrelated
  simulation methods;
- turns absence, transitive dependency failures, missing symbols, and version
  mismatches into actionable Rust errors;
- allows deployments and tests to select exact shared objects without changing
  global loader configuration; and
- keeps support tied to an audited ABI policy instead of whatever SDK happened
  to be present at build time.

Static or load-time linking would move an optional deployment concern into
every QDK build and process. A process-global native singleton was also
rejected: it would hide device and lifetime ownership, complicate tests, and
couple independent simulation workers. Dynamic loading costs a small explicit
discovery step and requires a maintained symbol/version policy; those are
appropriate costs for preserving QDK's existing CPU and cross-platform paths.

## Ownership

- `AvailabilityReport` is cloneable data containing the selected paths and
  observed versions. It owns no native resource.
- `Availability` owns the report and the private loader owner, ensuring that
  loaded libraries outlive their copied function pointers.
- `LoadedLibrary` owns one path and one `libloading::Library` guard.
- `CudaFunctions` and `CuTensorNetFunctions` are immutable typed function
  tables. Required tables are constructed atomically.
- `CuTensorNetApi` owns immutable shared-library guards plus immutable function
  tables, with no CUDA device or cuTensorNet execution handle. It is
  `Send + Sync`, so one discovery can be shared across threads; it shares no
  device context, stream or state.

Typical use is intentionally small:

```rust
let availability = qdk_cutensornet::discover()?;
let report = availability.report();

println!(
    "cuTensorNet {} loaded from {}",
    report.cutensornet_version,
    report.cutensornet_library.display()
);
```

Holding `availability` keeps both libraries and their function tables alive.
It does not reserve a GPU or imply that a later workload will fit in device
memory.

Code extending this crate should:

1. Treat discovery as an explicit capability check, not process
   initialization.
2. Keep the public API in QDK terms. Native library paths, symbols, pointers,
   and handles remain implementation details.
3. Add an ABI or runtime version only with retained header, symbol, layout,
   dependency, and hardware evidence. Do not broaden the allowlist merely
   because a newer library loads.
4. Preserve optionality. Ordinary tests and builds must never read SDK headers,
   call the binding generator, or require NVIDIA libraries.

## The MPS method

The MPS method is block ④ of the
[tensor-network design](../TensorNetworkDesign.md). It represents the state
of n qubits as a chain of site tensors $A^{[k]}$, one per qubit, joined by
bonds of dimension at most χ:

$$
\psi_{s_1 \dots s_n} \approx A^{[1]}_{s_1} A^{[2]}_{s_2} \cdots A^{[n]}_{s_n},
\qquad A^{[k]}_{s_k} \in \mathbb{C}^{\chi_{k-1} \times \chi_k},\ \chi_k \le \chi .
$$

Applying a two-qubit gate grows a bond; cuTensorNet truncates it back to χ by
an SVD that drops the smallest singular values. Memory and time grow with χ,
not with $2^n$, which is what makes circuits with low entanglement tractable.

It answers three queries of a Base program, for the Python API:

| Query         | Entry point                                         | Result                                                    |
| ------------- | --------------------------------------------------- | --------------------------------------------------------- |
| `Sample`      | `run_mps_shots`                                     | One list of output records per shot, as `run_qir` returns |
| `Expectation` | `evaluate_state_queries`, `StateQuery::Expectation` | ⟨ψ\|O\|ψ⟩ / ⟨ψ\|ψ⟩ for a Pauli sum O                      |
| `Cost`        | `evaluate_state_queries`, `StateQuery::Cost`        | The bond reached, state bytes and peak workspace bytes    |

ψ is $U|0\dots0\rangle$, the state before the terminal measurements. The
method owns the numerics; [the execution core](../simulators/src/execution/README.md)
(③) owns programs, shots and output records, and
[`tensornet::Mps`](../tensornet/README.md) (②) describes the shape of the
chain.

### Three layers

`execution` orchestrates an MPS request from program to results; `simulation`
implements the current cuTensorNet-shaped MPS lifecycle in host-portable Rust;
`library::simulation` supplies the Linux x86-64 native calls.

```text
execution.rs          orchestrates a request: host checks,    every platform
                      one session, ③ walks, results
   │ Circuit, ExecutionPolicy, SamplingRequest, PauliSum
   ▼
simulation            cuTensorNet-shaped MPS lifecycle         every platform
                      over four private test seams
  circuit             gate set, Circuit, state readouts
  policy              χ, cutoffs, SVD, gauge, workspace budget
  resources           device, stream, handle      ── SessionApi
  memory_workspace    device buffers, workspaces  ── MemoryWorkspaceApi
  mps_execution       one state's lifecycle       ── MpsExecutionApi
                      create → apply gates → finalize (χ) → compute → read → close
  sampler             sampler over the computed MPS ── SamplerApi
  query               Expectation results
  consumer            ③ region consumer (batch strategy)
  error               errors below the entry
  ffi                 Complex64Abi, the complex ABI value, and its checks
  branch              test only: project mid-circuit and continue
   │ the four traits
   ▼
library::simulation   Linux x86-64 only
  simulation.rs       the four traits for CuTensorNetApi, through ①'s function tables
  mps_session         MpsSession: the policy and SessionResources of one device
```

- _Host-portable_ says where this Rust logic compiles and is tested; it does
  not make the lifecycle _backend-neutral_.
- Every file lives in `src/library/simulation/`, but `src/simulation.rs`
  includes all of them except `simulation.rs` itself and `mps_session.rs` by
  `#[path]`, outside the Linux x86-64 gate on `library`. So `simulation`
  compiles on every platform, and its host tests run without a GPU.
- `CuTensorNetApi` owns the library guards and function tables, but no
  stream, handle, state or sampler; the lifecycle owners create and close
  those ([Session, execution and ownership](#session-execution-and-ownership)).
- `src/library/simulation.rs` is the only MPS layer that invokes the loaded
  CUDA and cuTensorNet operations.
- The four traits are test seams shaped around the cuTensorNet operations
  this implementation needs. They are not the backend-neutral interface of
  §4.2 of the [tensor-network design](../TensorNetworkDesign.md); that
  interface is extracted when a second backend arrives.

### One call, end to end

In one line: the entry runs Base programs with the MPS method, and `Sample`
uses the batch strategy. Every query starts from one MPS:

```text
Base program ─1─► Circuit     ③ walk on the host; rejects feedforward, reset, a 2nd region
             ─2─► one MPS ψ   GPU (MpsSession)
                    ├─ Expectation ─► read from ψ;  Cost ─► report the call's resources
                    └─ Sample ─► draw N rows from ψ
                                 ─3─► replay each row through ③ (shot results)
```

- The batch strategy is how `Sample` works: one state gives every shot.
- Base only applies to every query: step 1 rejects any Adaptive feature.
  `Expectation` stays Base-only by design, as in CUDA-Q's `observe`.
  Adaptive `Sample` is M1's `mps-a`, with the
  [incremental strategy](#adaptive-programs-the-incremental-strategy).

In detail, ⑤ calls the entry, and the entry calls ③ as a library, twice. The
GPU work happens in between, outside ③:

```text
⑤ tensornetwork_qir
  │
  ▼
④ run_mps_shots / evaluate_state_queries
  │
  ├─1─► ③ drive_prepared_shot(program, CircuitPreparationConsumer)     host, once
  │       the consumer records the gates into a Circuit and rejects a
  │       second region, feedforward and reset; qubits, shots and seeds
  │       are checked too                       program error, before discovery
  │
  ├─2─► discover()                              environment error from here on
  │     MpsSession::new(policy)                 stream + handle on one device
  │       MpsExecution::new                     |0…0⟩ state, workspace
  │         apply gates, finalize as an MPS (χ), compute once
  │         Sample:      sampler -> rows of bits
  │         Expectation: one native expectation per Pauli sum, normalized
  │       MpsExecution::close                   every native object released
  │
  ├─3─► ③ drive_prepared_shot(program, CuTensorNetMpsConsumer(row i))  host, per shot
  │       Sample only; the consumer answers each measurement from row i
  │       and ③ writes the output records
  │
  └───► MpsSession::close
```

Neither consumer touches the GPU, so ③ does not yet drive cuTensorNet. In
M1's `mps-a` (adaptive `Sample`), a consumer applies each region to the live
MPS and draws each measurement from it, so feedforward works; M2's `query`
moves the queries into ③.

Everything that depends only on the program is checked on the host before
`discover()`, so bad input is a _program error_ and never touches the GPU.
Discovery, native and cleanup failures are _environment errors_.
`MpsExecutionError::is_environment_error` tells them apart; the Python API
raises `ValueError` and `OSError` respectively. When execution and its
cleanup both fail, the error reports both.

### Sampling: the batch strategy

A Base program measures only at the end, so every shot samples the same
state. The method evolves ψ once and asks cuTensorNet's sampler for all the
shots at once, then hands each row to the execution core:

```text
              ┌──────────────── one MpsSession ────────────────┐
program ──► circuit ──► ψ (MPS, χ) ──► sampler ──► S[shots × m] bits
              └────────────────────────────────────────────────┘
                                                     │ row i
                                                     ▼
            drive_prepared_shot(program, CuTensorNetMpsConsumer(row i))
                                                     │
                                                     ▼
                                       output records of shot i
```

`CuTensorNetSampleMatrix` holds one column per distinct measured qubit. The
consumer implements ③'s `RegionConsumer`: it applies no gates, answers each
measurement from its row, and rejects what one sampled state cannot
represent, which is a second region, a region after a measurement, reset, and
a measurement after reset. The execution core still walks the program and
writes the records, so results have the same shape as every other simulator's.

Seeds are derived on the host: `seed` seeds a generator that draws one
positive 32-bit value used both as the sampler's path seed and as its sample
seed. Without `seed`, the value is drawn from entropy.

### Adaptive programs: the incremental strategy

The batch strategy needs a Base program. In an Adaptive program a later gate
can depend on an earlier outcome (feedforward), so shots no longer share one
state. M1's `mps-a` samples them with the incremental strategy the design
describes: shots run one at a time, ③ walks each one, and the consumer holds
the live MPS on the GPU.

```text
for each shot (fresh |0…0⟩, same MpsSession):
  ③ drive_prepared_shot(program, incremental MPS consumer)
     execute_region(R) ─► apply R's gates to the MPS, truncate to χ
     measure(q)        ─► p₀ = ⟨ψ|Π₀(q)|ψ⟩ / ⟨ψ|ψ⟩     from the MPS
                          r ~ Bernoulli(1 − p₀)       the shot's random stream
                          ψ ← Π_r(q) ψ / √p_r          project and renormalize
                          return r                    ③ feeds r forward
     reset(q)          ─► measure(q), then X on outcome 1
```

Each outcome is drawn conditioned on the earlier ones, so a shot's outcomes
have the program's distribution:

$$
P(r_1, \dots, r_M) = \prod_{k=1}^{M} P(r_k \mid r_{<k}) .
$$

What changes in this crate:

- The entry walks each shot once, with the consumer above, instead of
  walking once to build a circuit and again to replay rows; the host checks
  stop rejecting feedforward and reset for `Sample`.
- The consumer borrows one `MpsExecution` per shot from the session, so the
  session is reused across shots, one execution at a time, as today.
- The per-measurement step exists already as test-only code, used by the A100
  qualification to measure mid-circuit and continue: `compute_branch_masses`
  computes p₀ and p₁ from two projector expectations, and `apply_projection`
  projects the MPS. `mps-a` moves that step behind `RegionConsumer::measure`.

③ supplies the shot's random stream and records each outcome's probability
(M1's `ctr`). Base programs keep the batch strategy, which samples every shot
from one state.

### Expectation and Cost

For each Pauli sum $O = c_0 I + \sum_j c_j P_j$, the identity coefficient is
added on the host and the rest is one native expectation on the computed MPS:

$$
\langle O \rangle = c_0 + \frac{\langle \psi | O - c_0 I | \psi \rangle}{\langle \psi | \psi \rangle} .
$$

The division matters: truncation discards weight, so the MPS is not
normalized. Non-finite values and a nonpositive norm are rejected. Each
expectation's descriptors and buffers are released before the next one, so
device memory does not grow with the number of observables.

A call computes exactly one MPS, so `Cost` describes that state: the largest
bond it reached, at most χ; the bytes of its site tensors; and the largest
workspace any step requested. `Cost` with `Sample` is not available yet.

### Session, execution and ownership

```text
Arc<CuTensorNetApi>                 shared, immutable: library guards and function tables
  └─ MpsSession                     !Send, !Sync; one per call
       ├─ ExecutionPolicy           validated: fp64, GESVD, SIMPLE gauge, χ, cutoffs, budget
       ├─ SessionResources          device, CUDA stream, cuTensorNet handle
       └─ MpsExecution (borrowed)   one state: workspaces, operators, device buffers
            └─ PreparedSampler      one native sampler
```

- Each owner releases its children before its parent: sampler, then
  execution, then session. `close` releases every object on every path and
  reports the failure; `Drop` is a non-panicking fallback for paths that
  return early, not an error report.
- `MpsExecution` borrows the session's API, handle and stream. A session can
  serve several executions in sequence, never two at once.
- Gate matrices come from ③'s shared operator table, so every simulator
  applies the same matrices. The realized chain is checked against the
  `tensornet::Mps` the method asked for: no bond above χ, a qubit on every
  site.

The policy is fixed by the entry:

| Setting         | Value                          | Why                                                           |
| --------------- | ------------------------------ | ------------------------------------------------------------- |
| Precision       | fp64                           | the only qualified precision                                  |
| SVD, gauge      | GESVD, SIMPLE                  | the only qualified combination                                |
| χ               | 128, or `max_bond_dimension`   | the user's accuracy and memory trade-off                      |
| Cutoffs         | absolute 10⁻¹⁰, relative 10⁻¹⁶ | drop singular values that carry no weight                     |
| Workspace limit | at most 64 GiB                 | a request beyond it is an error, not an out-of-memory failure |

`ExecutionPolicy::validate` rejects anything else; qualifying another
configuration takes a new A100 qualification case first.

### Threading and processes

Parallelism comes from independent owners, not from sharing one native state.

- Discover once per process and share only `Arc<CuTensorNetApi>`. It holds no
  device, stream or state, so it is `Send + Sync`.
- Create, use and close each `MpsSession` on one thread. A `PhantomData<Rc<()>>`
  in `SessionResources` makes it `!Send` and `!Sync`, because cuTensorNet
  handles are tied to the CUDA device current when they were created, and
  states, workspaces and samplers are mutated by every call.
- Treat libraries, handles, streams and device pointers as process-local.
  Each process discovers on its own; choose GPUs per process before discovery.
  Fork safety has not been audited: do not use discovered libraries or
  sessions in a child after `fork()`.

### Test doubles and qualification

The native calls sit behind private injection traits (`SessionApi`,
`MemoryWorkspaceApi`, `SamplerApi`, `MpsExecutionApi`). `src/library/simulation.rs`
implements them with cuTensorNet; host tests implement them with test doubles
(`TestDoubleSessionApi`, `TestDoubleSamplerApi`, and the one in
`src/library/simulation/mps_execution/tests.rs`) that record call order and
inject failures. They check that every created object is released on every
path and that errors are reported as described above. They run on every host
without a GPU ([Three layers](#three-layers)).

Numbers come from the A100 qualification in
`src/library/simulation/mps_execution/qualification.rs`: `#[ignore]`d tests
that compare the MPS on a real GPU with closed-form amplitudes, pinned
reference values and the QDK's sparse simulator.

### Extending the method

`src/execution.rs` opens with the steps to add a state method or a query: the
variant, host validation before discovery, the session method that closes on
every path, and the Python encoding. Keep new native calls behind an injection
trait, give each native object one owner that closes it, and add the symbol to
`scripts/cutensornet-symbols.txt` before calling it.

## Validation

### CPU-only and no-SDK validation

```bash
cargo test -p qdk_cutensornet
cargo clippy --target x86_64-unknown-linux-gnu -p qdk_cutensornet --all-targets -- -D warnings
```

The loader is gated to Linux x86-64, so on other hosts its tests do not
compile; the `src/generator.rs` unit tests, the MPS method's host tests and the
non-x86-64 paths still run there. The second command type-checks the gated
code from any host. The MPS host tests run against
[test doubles](#test-doubles-and-qualification) and need no GPU. On Linux
x86-64, the focused loader tests verify:

- invalid relative, missing, and directory overrides fail before loading;
- every missing required cuTensorNet and CUDA symbol is reported by name;
- absence of optional `cutensornetGetLastError` does not reject discovery;
- malformed shared objects and exhausted candidate lists retain actionable
  context;
- CUDA version-probe failures preserve both status and copied native text;
- only the audited runtime values are accepted;
- known and unknown native status values remain distinguishable; and
- the cuTensorNet bindings and loader match the manifest's required and
  optional symbols, and the CUDA Runtime inventory remains aligned with its
  handwritten function table.

On every host, `checked_in_loader_matches_freshly_generated_output` checks that
the checked-in loader matches its generator. Compile-time assertions separately
verify the selected opaque pointer sizes, represented enum sizes/alignments,
complex element layout, and constants.

### Real-library validation

`tests/availability.rs` contains an ignored discovery test for the audited CUDA
Runtime 12.9 and cuTensorNet 2.13 installation. It opens and reports the real
libraries but deliberately performs no device selection, allocation, handle
creation, or GPU work:

```bash
cargo test -p qdk_cutensornet --test availability -- --ignored --nocapture
```

This test is native-environment evidence, not a replacement for CPU-only
tests.

The MPS method's numerical qualification is also `#[ignore]`d. It needs an
NVIDIA GPU (it was qualified on an A100) and runs every ignored test of the
crate:

```bash
cargo test -p qdk_cutensornet -- --ignored
```

On a CUDA host, `scripts/validate-on-cuda-host.sh --qualification` runs the
same suite after the loader checks, and fails if no qualification test ran;
see [Validating the FFI surface](scripts/README.md#validating-the-ffi-surface).

### Deterministic binding validation

Checked-in generated outputs are validated for reproducibility and freshness;
see [Validating the FFI surface](scripts/README.md#validating-the-ffi-surface).

## Binding provenance

The checked-in cuTensorNet declarations are generated from NVIDIA cuQuantum
26.06.0 for CUDA 12:

- artifact: `cuquantum-linux-x86_64-26.06.0.17_cuda12-archive.tar.xz`
- artifact SHA-256: `4c37aa346fab9023d985e79667b047e13a0c0f9b9fea7dfca453979b331c8f77`
- `cutensornet.h` SHA-256: `f70f31595c3c7b44682a7e4bdcd468504615983a4ec628f519cf18f0036a4687`
- bindgen CLI: 0.72.1
- clang: Ubuntu 14.0.0-1ubuntu1.1
- full reference output SHA-256: `8921d1acf0ff6d384a793893e92e10cadc850dfb29a0312726c31c4d692c3d7a`
- reduced output SHA-256: `2252b5ea29af1bf8ed5eb2d8268530ed289647791b9fa04bdc53c8599ee956da`
- reduced output line count: 882

Changing the selected functions or types changes the reduced output without
changing the SDK inputs. The full-reference hash depends on the headers and
pinned generation tools/options, not the reduced selection; its check remains
unchanged when only that selection changes.

The source artifact is identified by NVIDIA's `redistrib_26.06.0.json`.
NVIDIA headers, archives, and binaries are not stored in this repository. See
the NVIDIA cuQuantum SDK license distributed with that artifact for the source
material's terms.

## Regeneration

Generation is an explicit maintainer operation, not an ordinary Cargo build
step. The two maintainer-run generators are:

1. `scripts/generate-bindings.sh`, which generates
   `src/bindings/v2_13.rs`.
2. `cargo run -p qdk_cutensornet --bin generate-loader`, which generates
   `src/library/symbols.rs` and `src/library/symbols/*.rs`.

See [cuTensorNet FFI generation](scripts/README.md) for the SDK archive,
prerequisites, selection rules, host requirements, generated versus handwritten
files, and validation details. After updating the selection inputs, run from
the repository root:

```bash
source/cutensornet/scripts/generate-bindings.sh \
  /path/to/cuquantum-linux-x86_64-26.06.0.17_cuda12-archive.tar.xz \
  source/cutensornet/src/bindings/v2_13.rs &&
cargo run -p qdk_cutensornet --bin generate-loader
```
