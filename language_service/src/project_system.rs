use std::path::PathBuf;

use crate::LanguageService;

impl qsc_project::FileSystem for LanguageService<'_> {
    type Entry = PathBuf;

    fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        Ok((self.read_file)(path.into()))
    }

    fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.list_directory)(path.into()))
    }
}
