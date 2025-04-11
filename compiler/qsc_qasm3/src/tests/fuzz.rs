// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::compile_qasm_best_effort;
use qsc::target::Profile;

#[test]
fn fuzz_2297() {
    let source = r#"gate s a{gate a,b{}b"#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2298() {
    let source = r#"gate y()a{gate a,b{}b"#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}
