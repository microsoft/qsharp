// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{path::PathBuf, sync::Arc};

use expect_test::Expect;
use qsc_project::{FileSystem, Manifest, StdFs};

pub fn check(project_path: PathBuf, expect: &Expect) {
    let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_path.push(PathBuf::from("tests/projects"));
    let mut absolute_project_path = root_path.clone();
    absolute_project_path.push(project_path.clone());
    let manifest = Manifest::load_from_path(absolute_project_path)
        .unwrap()
        .unwrap();
    let fs = StdFs;
    let mut project = fs.load_project(&manifest).unwrap();

    // remove the prefix absolute path
    for (path, _contents) in project.sources.iter_mut() {
        let new_path = PathBuf::from(path.to_string());
        let new_path = new_path.strip_prefix(&root_path).unwrap().to_string_lossy();
        let new_path = new_path.replace(std::path::MAIN_SEPARATOR, "/");
        *path = Arc::from(new_path);
    }

    project.sources.sort();

    expect.assert_eq(&format!("{project:#?}"));
}
