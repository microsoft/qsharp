// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod defer_meas;
mod unreachable_code_check;

pub use defer_meas::defer_measurements;
pub use unreachable_code_check::{
    check_unreachable_blocks, check_unreachable_callable, check_unreachable_instrs,
};
