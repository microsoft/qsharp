//! Hand-audited CUDA Runtime 12 ABI declarations.
//!
//! The function signatures, types, and constants below are manually
//! transcribed from the NVIDIA CUDA Toolkit 12.9 header closure rooted at
//! `cuda_runtime_api.h`. This file is not generated. The `12` in the file name
//! denotes CUDA Runtime major version 12, not the number of functions.
//!
//! When widening this surface, update the `CudaFunctions` table and
//! `resolve_cuda_functions` in `src/library.rs`, the `CUDART_REQUIRED_SYMBOLS`
//! test inventory in `src/lib.rs`, and any affected ABI assertions in
//! `src/bindings/mod.rs`.

use super::v2_13::cudaStream_t;
use std::ffi::{c_char, c_int, c_uint, c_void};

pub(crate) type CudaError = c_uint;
pub(crate) type CudaMemcpyKind = c_uint;

pub(crate) const CUDA_MEMCPY_HOST_TO_DEVICE: CudaMemcpyKind = 1;
pub(crate) const CUDA_MEMCPY_DEVICE_TO_HOST: CudaMemcpyKind = 2;
pub(crate) const CUDA_STREAM_NON_BLOCKING: c_uint = 1;

pub(crate) type CudaRuntimeGetVersionFn = unsafe extern "C" fn(*mut c_int) -> CudaError;
pub(crate) type CudaDriverGetVersionFn = unsafe extern "C" fn(*mut c_int) -> CudaError;
pub(crate) type CudaGetDeviceCountFn = unsafe extern "C" fn(*mut c_int) -> CudaError;
pub(crate) type CudaSetDeviceFn = unsafe extern "C" fn(c_int) -> CudaError;
pub(crate) type CudaGetErrorStringFn = unsafe extern "C" fn(CudaError) -> *const c_char;
pub(crate) type CudaMemGetInfoFn = unsafe extern "C" fn(*mut usize, *mut usize) -> CudaError;
pub(crate) type CudaMallocFn = unsafe extern "C" fn(*mut *mut c_void, usize) -> CudaError;
pub(crate) type CudaFreeFn = unsafe extern "C" fn(*mut c_void) -> CudaError;
pub(crate) type CudaMemcpyFn =
    unsafe extern "C" fn(*mut c_void, *const c_void, usize, CudaMemcpyKind) -> CudaError;
pub(crate) type CudaStreamCreateWithFlagsFn =
    unsafe extern "C" fn(*mut cudaStream_t, c_uint) -> CudaError;
pub(crate) type CudaStreamSynchronizeFn = unsafe extern "C" fn(cudaStream_t) -> CudaError;
pub(crate) type CudaStreamDestroyFn = unsafe extern "C" fn(cudaStream_t) -> CudaError;
