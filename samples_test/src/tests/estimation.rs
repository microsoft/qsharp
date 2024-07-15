// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::compile;
use qsc::SourceMap;

include!(concat!(env!("OUT_DIR"), "/estimation_test_cases.rs"));
