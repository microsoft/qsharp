use crate::LanguageService;
use async_trait::async_trait;
use std::{convert::Infallible, path::PathBuf};

#[derive(Debug)]
pub struct JSFileEntry {
    pub name: String,
    pub r#type: qsc_project::EntryType,
}

impl qsc_project::DirEntry for JSFileEntry {
    type Error = Infallible;

    fn entry_type(&self) -> Result<qsc_project::EntryType, Self::Error> {
        Ok(self.r#type)
    }

    fn entry_extension(&self) -> String {
        let parsed_as_path = PathBuf::from(&self.name);
        parsed_as_path
            .extension()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn entry_name(&self) -> String {
        self.name.clone()
    }

    fn path(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}

#[async_trait(?Send)]
impl qsc_project::FileSystemAsync for LanguageService<'_> {
    type Entry = JSFileEntry;
    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        Ok((self.read_file_callback)(path.into()).await)
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.list_directory)(path.into()).await)
    }
}
