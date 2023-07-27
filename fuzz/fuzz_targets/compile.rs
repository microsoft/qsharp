// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![no_main]

#[cfg(feature = "do_fuzz")]
use libfuzzer_sys::fuzz_target;
use qsc::{hir::PackageId, PackageStore, SourceMap};

fn compile(data: &[u8]) {
    if let Ok(fuzzed_code) = std::str::from_utf8(data) {
        thread_local! {
            static STORE_STD: (PackageStore, PackageId) = {
                let mut store = PackageStore::new(qsc::compile::core());
                let std = store.insert(qsc::compile::std(&store));
                (store, std)
            };
        }
        let sources = SourceMap::new([("fuzzed_code".into(), fuzzed_code.into())], None);
        STORE_STD.with(|(store, std)| {
            let mut _unit = qsc::compile::compile(store, &[*std], sources, qsc::PackageType::Lib);
        });
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
