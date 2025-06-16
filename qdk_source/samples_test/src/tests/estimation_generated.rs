
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This build-generated module contains tests for the samples in the /samples/estimation folder.
//! DO NOT MANUALLY EDIT THIS FILE. To regenerate this file, run `cargo check` or `cargo test` in the `samples_test` directory.

use super::compile;
use qsc::SourceMap;

#[allow(non_snake_case)]
#[test]
fn compile_Dynamics() {
    compile(
        SourceMap::new(
            vec![("Dynamics.qs".into(), include_str!("../../../../../samples/estimation/Dynamics.qs").into())],
            None,
        )
    );
}

#[allow(non_snake_case)]
#[test]
fn compile_EkeraHastadFactoring() {
    compile(
        SourceMap::new(
            vec![("EkeraHastadFactoring.qs".into(), include_str!("../../../../../samples/estimation/EkeraHastadFactoring.qs").into())],
            None,
        )
    );
}

#[allow(non_snake_case)]
#[test]
fn compile_Precalculated() {
    compile(
        SourceMap::new(
            vec![("Precalculated.qs".into(), include_str!("../../../../../samples/estimation/Precalculated.qs").into())],
            None,
        )
    );
}

#[allow(non_snake_case)]
#[test]
fn compile_ShorRE() {
    compile(
        SourceMap::new(
            vec![("ShorRE.qs".into(), include_str!("../../../../../samples/estimation/ShorRE.qs").into())],
            None,
        )
    );
}
