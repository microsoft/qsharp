// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{path::Path, sync::Arc};

use qsc::{qasm::parser::ast::StmtKind, LanguageFeatures};
use qsc_project::{FileSystemAsync, PackageGraphSources, PackageInfo, Project};
use rustc_hash::FxHashMap;

pub async fn load_project<T>(project_host: &T, doc_uri: &Arc<str>) -> Project
where
    T: FileSystemAsync + ?Sized,
{
    let mut loaded_files = FxHashMap::default();
    let mut pending_includes = vec![];
    let mut errors = vec![];

    // this is the root of the project
    // it is the only file that has a full path.
    // all other files are relative to this one.
    // and the directory above is the directory of the file
    // we need to combine the two to get the full path
    pending_includes.push(doc_uri.clone());

    while let Some(current) = pending_includes.pop() {
        if loaded_files.contains_key(&current) {
            // We've already loaded this include, so skip it.
            // We'll let the source resolver handle any duplicates.
            // and cyclic dependency errors.
            continue;
        }

        match project_host.read_file(Path::new(current.as_ref())).await {
            Ok((file, source)) => {
                loaded_files.insert(file, source.clone());

                let (program, _errors) = qsc::qasm::parser::parse(source.as_ref());

                let includes: Vec<Arc<str>> = program
                    .statements
                    .iter()
                    .filter_map(|stmt| {
                        if let StmtKind::Include(include) = &*stmt.kind {
                            Some(include.filename.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                for include in includes {
                    if include == "stdgates.inc".into() {
                        // Don't include stdgates.inc, as it is a special case.
                        continue;
                    }
                    if loaded_files.contains_key(&include) {
                        // We've already loaded this include, so skip it.
                        continue;
                    }

                    pending_includes.push(include);
                }
            }
            Err(e) => {
                errors.push(qsc::project::Error::FileSystem {
                    about_path: doc_uri.to_string(),
                    error: e.to_string(),
                });
            }
        }
    }

    let sources = loaded_files.into_iter().collect::<Vec<_>>();

    Project {
        package_graph_sources: PackageGraphSources {
            root: PackageInfo {
                sources,
                language_features: LanguageFeatures::default(),
                dependencies: FxHashMap::default(),
                package_type: None,
            },
            packages: FxHashMap::default(),
        },
        path: doc_uri.clone(),
        name: get_file_name_from_uri(doc_uri),
        lints: Vec::default(),
        errors,
        project_type: qsc_project::ProjectType::OpenQASM,
    }
}

fn get_file_name_from_uri(uri: &Arc<str>) -> Arc<str> {
    // Convert the Arc<str> into a &str and then into a Path
    let path = Path::new(uri.as_ref());

    // Extract the file name or return the original URI if it fails
    path.file_name()
        .and_then(|name| name.to_str().map(|s| s.to_string().into()))
        .map_or_else(|| uri.clone(), |f| f)
}
