// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::output::Receiver;
use crate::{eval_stmt, Env};
use qsc_ast::ast::CallableDecl;
use qsc_passes::globals::GlobalId;
use std::collections::HashMap;
use std::string::String;

use qsc_frontend::compile::{self, PackageStore};
use qsc_frontend::incremental::{Compiler, Fragment};
use std::io::Cursor;
use std::io::Write;

use num_bigint::BigUint;
use num_complex::Complex64;
use ouroboros::self_referencing;
use qsc_frontend::compile::compile;
use qsc_passes::globals::extract_callables;

use crate::output;

struct CursorReceiver<'a> {
    cursor: &'a mut Cursor<Vec<u8>>,
}

impl<'a> CursorReceiver<'a> {
    pub fn new(cursor: &'a mut Cursor<Vec<u8>>) -> Self {
        Self { cursor }
    }
    fn dump(&mut self) -> String {
        let v = self.cursor.get_mut();
        let s = match std::str::from_utf8(v) {
            Ok(v) => v.to_owned(),
            Err(e) => format!("Invalid UTF-8 sequence: {e}"),
        };
        v.clear();
        s
    }
}

impl<'a> Receiver for CursorReceiver<'a> {
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), output::Error> {
        writeln!(self.cursor, "STATE:").map_err(|_| output::Error)?;
        for (id, state) in state {
            writeln!(self.cursor, "|{}⟩: {}", id.to_str_radix(2), state)
                .map_err(|_| output::Error)?;
        }
        Ok(())
    }

    fn message(&mut self, msg: String) -> Result<(), output::Error> {
        writeln!(self.cursor, "{msg}").map_err(|_| output::Error)
    }
}

#[self_referencing]

pub struct ExecutionContext {
    store: PackageStore,
    package: compile::PackageId,
    #[borrows(store)]
    #[covariant]
    compiler: Compiler<'this>,
    #[borrows(store)]
    #[not_covariant]
    globals: HashMap<GlobalId, &'this CallableDecl>,
    env: Option<Env>,
    cursor: Cursor<Vec<u8>>,
    #[borrows(mut cursor)]
    #[not_covariant]
    out: CursorReceiver<'this>,
}

pub struct Interpreter {
    context: ExecutionContext,
}

impl Interpreter {
    #[must_use]
    pub fn new(nostdlib: bool, sources: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut store = PackageStore::new();

        let dependencies = if nostdlib {
            vec![]
        } else {
            vec![store.insert(compile::std())]
        };

        let mut v: Vec<_> = dependencies.clone();

        let package = store.insert(compile(&store, dependencies, sources, ""));
        v.push(package);
        let sources: [&str; 0] = [];

        let r = store.insert(compile(&store, [], sources, ""));

        let context = ExecutionContextBuilder {
            store,
            package: r,
            compiler_builder: |store| Compiler::new(store, v),
            globals_builder: extract_callables,
            env: None,
            cursor: Cursor::new(Vec::<u8>::new()),
            out_builder: |cursor| CursorReceiver::new(cursor),
        }
        .build();
        Self { context }
    }

    pub fn line(&mut self, line: impl AsRef<str>) -> (String, String, String) {
        self.context.with_mut(|fields| {
            let fragment = fields.compiler.compile_fragment(line);
            match fragment {
                Ok(a) => match a {
                    Fragment::Stmt(stmt) => {
                        let mut env = fields.env.take().unwrap_or(Env::empty());
                        let result = eval_stmt(
                            stmt,
                            fields.store,
                            fields.globals,
                            fields.compiler.resolutions(),
                            *fields.package,
                            &mut env,
                            fields.out,
                        );
                        let _ = fields.env.insert(env);
                        let output = fields.out.dump();

                        match result {
                            Ok(v) => (format!("{v}"), output, String::new()),
                            Err(e) => (String::new(), output, format!("{e}")),
                        }
                    }
                    Fragment::Callable(decl) => {
                        let id = GlobalId {
                            package: *fields.package,
                            node: decl.name.id,
                        };
                        fields.globals.insert(id, decl);
                        (String::new(), String::new(), String::new())
                    }
                },
                Err(errors) => {
                    let e = errors
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<String>();
                    (String::new(), String::new(), e)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn members_from_namespaced_sources_are_in_context() {
        let source = indoc! { r#"
            namespace Test {
                function Hello() : String {
                    "hello there..."
                }

                operation Main() : String {
                    Hello()
                }
            }"#};

        let mut interpreter = Interpreter::new(false, [source]);
        let (value, _, _) = interpreter.line("Test.Hello()");
        assert_eq!("hello there...", value);
        let (value, _, _) = interpreter.line("Test.Main()");
        assert_eq!("hello there...", value);
    }

    #[test]
    fn multiple_namespaces_are_loaded_from_sources_into_eval_context() {
        let source = indoc! { r#"
            namespace Test {
                function Hello() : String {
                    "hello there..."
                }
            }
            namespace Test2 {
                open Test;
                operation Main() : String {
                    Hello()
                }
            }"#};

        let mut interpreter = Interpreter::new(false, [source]);
        let (value, _, _) = interpreter.line("Test.Hello()");
        assert_eq!("hello there...", value);
        let (value, _, _) = interpreter.line("Test2.Main()");
        assert_eq!("hello there...", value);
    }
}
