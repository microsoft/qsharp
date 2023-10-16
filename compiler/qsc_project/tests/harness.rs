use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use expect_test::Expect;
use miette::{Context, IntoDiagnostic};
use qsc_project::Project;

pub fn check(project_path: PathBuf, expect: &Expect) {
    let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_path.push(PathBuf::from(format!("tests/projects")));
    let mut absolute_project_path = root_path.clone();
    absolute_project_path.push(project_path.clone());
    let mut project = Project::load_from_path(absolute_project_path, |x| read_source(x)).unwrap();

    // remove the prefix absolute path
    for (path, _contents) in project.sources.iter_mut() {
        let new_path = PathBuf::from(path.to_string());
        let new_path = new_path.strip_prefix(&root_path).unwrap();
        *path = Arc::from(new_path.to_str().unwrap_or_default());
    }

    expect.assert_eq(&format!("{project:#?}"));
}

fn read_source(path: impl AsRef<Path>) -> miette::Result<(Arc<str>, Arc<str>)> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("could not read source file `{}`", path.display()))?;

    Ok((path.to_string_lossy().into(), contents.into()))
}
