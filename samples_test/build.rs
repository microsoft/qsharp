// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    env,
    fs::{read_dir, File},
    io::Write,
    path::Path,
};

fn main() {
    println!("cargo::rerun-if-changed=../samples/algorithms/");
    // Iterate through the samples folder and create a test for each file
    let mut paths =
        read_dir("../samples/algorithms/").expect("folder should exist and be readable");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should be set");
    let dest_path = Path::new(&out_dir).join("test_cases.rs");
    let mut f = File::create(dest_path).expect("files should be creatable in OUT_DIR");

    while let Some(Ok(dir_entry)) = paths.next() {
        let path = &dir_entry.path();
        let file_name = path
            .file_name()
            .expect("file name should be separable")
            .to_str()
            .expect("file name should be valid");
        let file_stem = path
            .file_stem()
            .expect("file name should be separable")
            .to_str()
            .expect("file name should be valid");
        assert!(
            !file_stem.contains(' '),
            "file name `{file_name}` should not contain spaces"
        );
        let file_stem_upper = file_stem.to_uppercase();

        writeln!(
            f,
            r#"
            #[allow(non_snake_case)]
            fn {file_stem}_src() -> SourceMap {{
                SourceMap::new(
                    vec![("{file_name}".into(), include_str!("../../../../../samples/algorithms/{file_name}").into())],
                    None,
                )
            }}

            #[allow(non_snake_case)]
            #[test]
            fn run_{file_stem}() {{
                let output = compile_and_run({file_stem}_src());
                // This constant must be defined in `samples_test/src/tests.rs` and
                // must contain the output of the sample {file_name}
                {file_stem_upper}_EXPECT.assert_eq(&output);
            }}

            #[allow(non_snake_case)]
            #[test]
            fn debug_{file_stem}() {{
                let output = compile_and_run_debug({file_stem}_src());
                // This constant must be defined in `samples_test/src/tests.rs` and
                // must contain the output of the sample {file_name}
                {file_stem_upper}_EXPECT_DEBUG.assert_eq(&output);
            }}
            "#
        )
        .expect("writing to file should succeed");
    }
}
