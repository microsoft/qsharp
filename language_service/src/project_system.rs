use std::{
    future::{self, Future},
    path::PathBuf,
    pin::Pin,
};

use crate::LanguageService;

impl qsc_project::FileSystem for LanguageService<'_> {
    type Entry = PathBuf;

    fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        Ok((self.read_file)(path.into()))
    }

    fn list_directory(
        &self,
        path: &std::path::Path,
    ) -> Pin<Box<dyn Future<Output = miette::Result<Vec<Self::Entry>>>>> {
        Box::pin(future::ready(Ok((self.list_directory)(path.into()))))
    }
}
