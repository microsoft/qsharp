// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![no_main]

allocator::assign_global!();

#[cfg(feature = "do_fuzz")]
use libfuzzer_sys::fuzz_target;

fn compile(data: &[u8]) {
    if let Ok(fuzzed_code) = std::str::from_utf8(data) {
        let mut resolver = qsc::qasm::io::InMemorySourceResolver::from_iter([]);
        let _ = qsc::qasm::parser::parse_source(fuzzed_code, "fuzz.qasm", &mut resolver);
    }
}

#[cfg(feature = "do_fuzz")]
fuzz_target!(|data: &[u8]| {
    compile(data);
});

#[cfg(not(feature = "do_fuzz"))]
#[no_mangle]
pub extern "C" fn main() {
    compile(&[]);
}
