// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::output::Receiver;
use crate::{eval_stmt, Env};
use qsc_ast::ast::CallableDecl;
use qsc_passes::globals::GlobalId;
use std::collections::HashMap;
use std::string::String;

use qsc_frontend::compile::{self, CompileUnit, PackageStore};
use qsc_frontend::incremental::{Compiler, Fragment};
use std::io::Cursor;
use std::io::Write;

use crate::output;
use miette::Diagnostic;
use num_bigint::BigUint;
use num_complex::Complex64;
use ouroboros::self_referencing;
use qsc_frontend::compile::compile;
use qsc_passes::globals::extract_callables;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Eval(crate::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(qsc_frontend::compile::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Incremental(qsc_frontend::incremental::Error),
}

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
        s.trim().to_string()
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

pub struct InterpreterResult {
    pub output: String,
    pub value: String,
    pub errors: Vec<Error>,
}

impl InterpreterResult {
    #[must_use]
    pub fn new(value: String, output: String, errors: Vec<Error>) -> Self {
        Self {
            output,
            value,
            errors,
        }
    }
}

pub struct Interpreter {
    context: ExecutionContext,
}

impl Interpreter {
    pub fn test_input(
        nostdlib: bool,
        sources: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> CompileUnit {
        let mut store = PackageStore::new();

        let mut session_deps: Vec<_> = vec![];

        if !nostdlib {
            session_deps.push(store.insert(compile::std()));
        }

        // create a package with all defined dependencies for the session
        compile(&store, session_deps.clone(), sources, "")
    }

    /// # Panics
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    /// Use `Interpreter::test_input` to test compilation.
    pub fn new(nostdlib: bool, sources: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut store = PackageStore::new();

        let mut session_deps: Vec<_> = vec![];

        if !nostdlib {
            session_deps.push(store.insert(compile::std()));
        }

        // create a package with all defined dependencies for the session
        let unit = compile(&store, session_deps.clone(), sources, "");

        assert!(
            unit.context.errors().is_empty(),
            "Compilation failed: {:?}",
            unit.context.errors()
        );
        let basis_package = store.insert(unit);
        session_deps.push(basis_package);

        // create a package with no dependencies for the session
        let sources: [&str; 0] = [];
        let session_package = store.insert(compile(&store, [], sources, ""));

        let context = ExecutionContextBuilder {
            store,
            package: session_package,
            compiler_builder: |store| Compiler::new(store, session_deps),
            globals_builder: extract_callables,
            env: None,
            cursor: Cursor::new(Vec::<u8>::new()),
            out_builder: |cursor| CursorReceiver::new(cursor),
        }
        .build();
        Self { context }
    }

    pub fn line(&mut self, line: impl AsRef<str>) -> Vec<InterpreterResult> {
        let mut results = vec![];
        self.context.with_mut(|fields| {
            let fragments = fields.compiler.compile_fragment(line);
            for fragment in fragments {
                match fragment {
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
                            Ok(v) => {
                                results.push(InterpreterResult::new(
                                    format!("{v}"),
                                    output,
                                    vec![],
                                ));
                            }
                            Err(e) => results.push(InterpreterResult::new(
                                String::new(),
                                output,
                                vec![crate::interactive::Error::Eval(e)],
                            )),
                        }
                    }
                    Fragment::Callable(decl) => {
                        let id = GlobalId {
                            package: *fields.package,
                            node: decl.name.id,
                        };
                        fields.globals.insert(id, decl);
                        results.push(InterpreterResult::new(String::new(), String::new(), vec![]));
                    }
                    Fragment::Error(errors) => {
                        let e = errors
                            .iter()
                            .map(|e| crate::interactive::Error::Incremental(e.clone()))
                            .collect();
                        results.push(InterpreterResult::new(String::new(), String::new(), e));
                    }
                }
            }
        });
        results
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
        let result = &interpreter.line("Test.Hello()")[0];
        assert_eq!("hello there...", result.value);
        let result = &interpreter.line("Test.Main()")[0];
        assert_eq!("hello there...", result.value);
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
        let result = &interpreter.line("Test.Hello()")[0];
        assert_eq!("hello there...", result.value);
        let result = &interpreter.line("Test2.Main()")[0];
        assert_eq!("hello there...", result.value);
    }

    #[test]
    fn nostdlib_is_omitted_correctly() {
        let sources: [&str; 0] = [];
        let mut interpreter = Interpreter::new(true, sources);
        let result = &interpreter.line("Message(\"_\")")[0];

        assert_eq!("", result.value);
        assert_eq!("", result.output);
        assert_eq!(
            "`Message` not found in this scope",
            result.errors[0].to_string()
        );
    }

    #[test]
    fn nostdlib_is_included_correctly() {
        let sources: [&str; 0] = [];
        let mut interpreter = Interpreter::new(false, sources);
        let result = &interpreter.line("Message(\"_\")")[0];

        assert_eq!("()", result.value);
        assert_eq!("_", result.output);
        assert_eq!(0, result.errors.len());
    }
}
