// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use expect_test::Expect;
use qsc_project::{
    key_for_package_ref, package_ref_from_key, Error, FileSystem, Manifest, PackageRef, Project,
    StdFs,
};
use rustc_hash::FxHashMap;

pub fn check(project_path: &PathBuf, expect: &Expect) {
    let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_path.push(PathBuf::from("src"));
    root_path.push(PathBuf::from("tests"));
    root_path.push(PathBuf::from("projects"));
    let mut absolute_project_path = root_path.clone();
    absolute_project_path.push(project_path);
    let manifest = Manifest::load_from_path(absolute_project_path)
        .expect("manifest should load")
        .expect("manifest should contain descriptor");
    let fs = StdFs;
    let mut project = fs
        .load_project(&manifest.manifest_dir, None)
        .expect("project should load");

    normalize(&mut project, &root_path);

    expect.assert_eq(&format!("{project:#?}"));
}

/// If the `Project` contains absolute paths, replace them with relative paths
/// so that running the tests on different machines produce the same results.
/// Some error messages may contain paths formatted into strings, in that case
/// we'll just replace the message with filler text.
fn normalize(project: &mut Project, root_path: &Path) {
    let pkg_graph = &mut project.package_graph_sources;

    normalize_pkg(&mut pkg_graph.root, root_path);

    let mut new_packages = FxHashMap::default();
    for (mut key, mut pkg) in pkg_graph.packages.drain() {
        remove_absolute_path_prefix_from_key(&mut key, root_path);

        normalize_pkg(&mut pkg, root_path);

        new_packages.insert(key, pkg);
    }

    pkg_graph.packages = new_packages;

    remove_absolute_path_prefix(&mut project.path, root_path);

    for err in &mut project.errors {
        match err {
            Error::NoSrcDir { path }
            | Error::ManifestParse { path, .. }
            | Error::GitHubManifestParse { path, .. } => {
                let mut str = std::mem::take(path).into();
                remove_absolute_path_prefix(&mut str, root_path);
                *path = str.to_string();
            }
            Error::FileSystem {
                about_path: path,
                error,
            } => {
                let mut str = std::mem::take(path).into();
                remove_absolute_path_prefix(&mut str, root_path);
                *path = str.to_string();
                *error = "REPLACED".to_string();
            }
            Error::Circular(s1, s2) | Error::GitHubToLocal(s1, s2) => {
                // These errors contain absolute paths which don't work well in test output
                *s1 = "REPLACED".to_string();
                *s2 = "REPLACED".to_string();
            }
            Error::GitHub(s) => {
                *s = "REPLACED".to_string();
            }
        }
    }
}

fn normalize_pkg(pkg: &mut qsc_project::PackageInfo, root_path: &Path) {
    for (path, _contents) in &mut pkg.sources {
        remove_absolute_path_prefix(path, root_path);
    }
    pkg.sources.sort();

    for key in pkg.dependencies.values_mut() {
        remove_absolute_path_prefix_from_key(key, root_path);
    }
}

fn remove_absolute_path_prefix(path: &mut Arc<str>, root_path: &Path) {
    let new_path = PathBuf::from(path.to_string());
    let new_path = new_path
        .strip_prefix(root_path)
        .unwrap_or_else(|_| {
            panic!(
                "prefix {} should be present in {}",
                root_path.display(),
                path
            )
        })
        .to_string_lossy();
    let new_path = new_path.replace(std::path::MAIN_SEPARATOR, "/");
    *path = Arc::from(new_path);
}

fn remove_absolute_path_prefix_from_key(key: &mut Arc<str>, root_path: &Path) {
    let def = package_ref_from_key(key);
    if let PackageRef::Path { path } = def {
        let mut path = path.into();
        remove_absolute_path_prefix(&mut path, root_path);
        *key = key_for_package_ref(&PackageRef::Path {
            path: path.to_string(),
        });
    }
}
