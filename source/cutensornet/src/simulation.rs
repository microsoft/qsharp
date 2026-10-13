//! Host-portable cuTensorNet MPS mechanics for block ④.
//!
//! `execution` orchestrates an MPS request from program to results;
//! `simulation` constructs, evolves, reads, samples and closes one MPS state;
//! `library::simulation` connects those mechanics to cuTensorNet:
//!
//! ```text
//! execution            a request: program → results          every platform
//!   ▼
//! simulation           cuTensorNet-shaped MPS lifecycle       every platform
//!                      over four private test seams
//!   ▼
//! library::simulation  the traits on cuTensorNet              Linux x86-64
//! ```
//!
//! "Host-portable" says where this Rust logic compiles and is tested; it does
//! not make the lifecycle backend-neutral. The four traits mirror the
//! cuTensorNet operations this implementation needs.
//!
//! The host-portable files live in `library/simulation/` and are included here
//! by `#[path]`, outside the Linux x86-64 gate on `library`, so their host tests,
//! which use test doubles of the native APIs, run on every platform. Only
//! `library::simulation` invokes the loaded CUDA and cuTensorNet operations; it
//! implements the injection traits declared in these files. The README's
//! "Three layers" section lists what each file does. Test-only items are
//! `#[cfg(test)]` at their definition. A new host-portable file under
//! `library/simulation/` needs a `#[path]` entry here.

#[cfg(test)]
#[path = "library/simulation/branch.rs"]
mod branch;
#[path = "library/simulation/circuit.rs"]
mod circuit;
#[path = "library/simulation/consumer.rs"]
mod consumer;
#[path = "library/simulation/error.rs"]
mod error;
#[path = "library/simulation/ffi.rs"]
mod ffi;
#[path = "library/simulation/memory_workspace.rs"]
pub(crate) mod memory_workspace;
#[path = "library/simulation/mps_execution.rs"]
mod mps_execution;
#[path = "library/simulation/policy.rs"]
mod policy;
#[path = "library/simulation/query.rs"]
mod query;
#[path = "library/simulation/resources.rs"]
pub(crate) mod resources;
#[path = "library/simulation/sampler.rs"]
mod sampler;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) use sampler::SamplerApi;

pub(super) use circuit::{Circuit, Gate, SimulationResult, UnitaryOperationConversionError};
#[cfg(test)]
pub(super) use circuit::{ProjectedCircuit, ProjectedOperation};
#[cfg(any(test, all(target_os = "linux", target_arch = "x86_64")))]
pub(super) use consumer::collect_sampled_shots;
pub(super) use consumer::{CuTensorNetMpsConsumerError, CuTensorNetSampleMatrix};
pub(super) use error::SimulationError;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(super) use ffi::Complex64Abi;
pub(super) use mps_execution::StateQueryResult;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(super) use mps_execution::{
    MpsExecutionApi, MpsTarget, OutputMetadata, StateF64Attribute, StateU32Configuration,
};
#[cfg(any(test, all(target_os = "linux", target_arch = "x86_64")))]
pub(super) use policy::ExecutionPolicy;
pub(super) use sampler::SamplingRequest;

use std::{ffi::c_void, ptr::NonNull};

pub(super) type OpaqueHandle = NonNull<c_void>;
pub(super) type Stream = NonNull<c_void>;
