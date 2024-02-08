// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::generate_docs;
use crate::compilation::Compilation;

#[test]
fn callable_unit_types() {
    let compilation = Compilation::new(
        &[],
        qsc::PackageType::Exe,
        qsc::target::Profile::Unrestricted,
    );
    generate_docs(&compilation);
}
