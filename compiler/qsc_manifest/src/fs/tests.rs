// use std::{ffi::OsStr, path::PathBuf, rc::Rc, sync::Arc};

// use super::TraversableFilesystem;
// #[derive(Debug)]
// struct MockFilesystem {
//     root: Entry,
// }
// impl MockFilesystem {
//     fn lookup(&self, path: PathBuf) -> Option<&Entry> {
//         let path = path.into_iter().map(|x| x.to_str().unwrap());
//         self.root.lookup(path)
//     }
// }

// #[derive(Debug)]
// enum Entry {
//     File { name: String, contents: String },
//     Directory(Directory),
// }

// impl crate::fs::Entry for MockFilesystemPath {
//     fn name_of_entry(&self) -> Arc<str> {
//         Arc::from(dbg!(self
//             .path
//             .file_name()
//             .unwrap()
//             .to_str()
//             .unwrap_or_default()))
//     }

//     fn path(&self) -> PathBuf {
//         self.path.clone()
//     }
// }

// impl Entry {
//     fn name(&self) -> &str {
//         match self {
//             Entry::File { ref name, .. } => name,
//             Entry::Directory(Directory { ref name, .. }) => name,
//         }
//     }
//     fn lookup<'a>(&self, mut path: impl Iterator<Item = &'a str>) -> Option<&Entry> {
//         let next = match path.next() {
//             Some(x) => x,
//             None => return Some(self),
//         };

//         match &self {
//             Entry::File { .. } => return None,
//             Entry::Directory(dir) => {
//                 for item in &dir.entries {
//                     if item.name() == next {
//                         return item.lookup(path);
//                     }
//                 }
//             }
//         }
//         None
//     }
//     fn file(name: impl Into<String>, contents: impl Into<String>) -> Self {
//         let name = name.into();
//         let contents = contents.into();
//         Entry::File { name, contents }
//     }
//     fn dir(name: impl Into<String>, entries: Vec<Entry>) -> Self {
//         let name = name.into();
//         Entry::Directory(Directory { name, entries })
//     }
// }

// #[derive(Debug)]
// struct Directory {
//     entries: Vec<Entry>,
//     name: String,
// }
// // impl Directory {
// //     fn lookup<T: AsRef<str>>(&self, mut path: impl Iterator<Item = T>) -> Option<Entry> {

// //     }
// // }

// #[derive(Debug)]
// struct MockFilesystemPath {
//     fs: Rc<MockFilesystem>,
//     path: PathBuf,
// }

// impl MockFilesystemPath {
//     fn pop(&mut self) -> bool {
//         self.path.pop()
//     }

//     fn construct_fs_path(&self, x: &Entry) -> MockFilesystemPath {
//         let mut path = self.path.clone();
//         match x {
//             Entry::File { name, .. } => path.set_file_name(name),
//             Entry::Directory(dir) => path.push(&dir.name),
//         }

//         MockFilesystemPath {
//             fs: self.fs.clone(),
//             path,
//         }
//     }

//     fn new(fs: MockFilesystem, path: impl Into<PathBuf>) -> Self {
//         Self {
//             fs: Rc::new(fs),
//             path: path.into(),
//         }
//     }
// }

// #[derive(Debug)]
// struct MockError(String);

// impl From<serde_json::Error> for MockError {
//     fn from(value: serde_json::Error) -> Self {
//         Self(format!("{value:?}"))
//     }
// }

// impl TraversableFilesystem for MockFilesystemPath {
//     type Error = MockError;
//     type Entry = MockFilesystemPath;

//     fn parent_dir(&self) -> Option<Self> {
//         let mut path = self.path.clone();
//         if path.pop() {
//             Some(MockFilesystemPath {
//                 path,
//                 fs: self.fs.clone(),
//             })
//         } else {
//             None
//         }
//     }

//     fn read_file_to_string(&self, file_entry: std::path::PathBuf) -> Result<String, Self::Error> {
//         let file = match self.fs.lookup(dbg!(file_entry)) {
//             Some(x) => x,
//             None => return Err(MockError("file lookup failed".into())),
//         };
//         match file {
//             Entry::File { contents, .. } => Ok(contents.clone()),
//             Entry::Directory(_) => Err(MockError("this was a dir".into())),
//         }
//     }

//     fn read_directory(&self) -> Result<Vec<Result<MockFilesystemPath, MockError>>, MockError> {
//         let file = match self.fs.lookup(dbg!(self.path.clone())) {
//             Some(x) => x,
//             None => return Err(MockError("148".into())),
//         };
//         match file {
//             Entry::File { .. } => Err(MockError("151".into())),
//             Entry::Directory(dir) => Ok(dir
//                 .entries
//                 .iter()
//                 .map(|x| Ok(self.construct_fs_path(x)))
//                 .collect()),
//         }
//     }
// }

// fn test_structure() -> MockFilesystem {
//     MockFilesystem {
//         root: Entry::dir(
//             "root",
//             vec![Entry::dir(
//                 "code",
//                 vec![
//                     Entry::file("foo.qs", "namespace Foo {}"),
//                     Entry::file("bar.qs", "namespace Bar {}"),
//                     Entry::dir(
//                         "nested",
//                         vec![
//                             Entry::file("baz.qs", "namespace Baz {}"),
//                             Entry::file("qsharp.json", "{}"),
//                             Entry::file("quux.qs", "namespace Quux {}"),
//                             Entry::dir(
//                                 "utils",
//                                 vec![Entry::file("utils.qs", "namespace Utils {}")],
//                             ),
//                         ],
//                     ),
//                 ],
//             )],
//         ),
//     }
// }

// #[test]
// fn find_manifest() {
//     let test = MockFilesystemPath::new(test_structure(), "code/nested/utils");
//     dbg!(test.manifest_file());
//     panic!();
// }
