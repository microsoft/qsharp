// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::CallableDecl;
use qsc_eval::evaluate;
use qsc_eval::output::Receiver;
use qsc_eval::Env;
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

pub struct CursorReceiver<'a> {
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
            Err(e) => format!("Invalid UTF-8 sequence: {}", e),
        };
        v.clear();
        s
    }
}

impl<'a> Receiver for CursorReceiver<'a> {
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), qsc_eval::output::Error> {
        writeln!(self.cursor, "STATE:").map_err(|_| qsc_eval::output::Error)?;
        for (id, state) in state {
            writeln!(self.cursor, "|{}⟩: {}", id.to_str_radix(2), state)
                .map_err(|_| qsc_eval::output::Error)?;
        }
        Ok(())
    }

    fn message(&mut self, msg: String) -> Result<(), qsc_eval::output::Error> {
        writeln!(self.cursor, "{msg}").map_err(|_| qsc_eval::output::Error)
    }
}

#[self_referencing]

pub(crate) struct ExecutionContext {
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

impl ExecutionContext {
    pub fn eval(&mut self, line: String) -> (String, String, String) {
        let fragment = self.with_compiler_mut(|compiler| compiler.compile_fragment(&line));

        if let Err(errors) = fragment {
            let e = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("");
            return (String::new(), String::new(), e);
        };
        let fragment = fragment.unwrap();
        if let Fragment::Callable(decl) = fragment {
            let id = GlobalId {
                package: *self.borrow_package(),
                node: decl.name.id,
            };
            self.with_globals_mut(|globals| {
                globals.insert(id, decl);
            });
            return (String::new(), String::new(), String::new());
        };

        if let Fragment::Stmt(stmt) = fragment {
            let (result, s) = self.with_mut(|fields| {
                let env = fields.env.take().unwrap_or(Env::empty());
                let (result, new_env) = evaluate(
                    stmt,
                    fields.store,
                    fields.globals,
                    fields.compiler.resolutions(),
                    *fields.package,
                    env,
                    fields.out,
                );
                let output = fields.out.dump();
                *(fields.env) = Some(new_env);
                (result, output)
            });
            match result {
                Ok(v) => {
                    return (format!("{v}"), s, String::new());
                }
                Err(e) => {
                    return (String::new(), s, format!("{e}"));
                }
            }
        };

        unreachable!("Fragment is not callable or statement")
    }
}

pub(crate) fn create_context() -> ExecutionContext {
    let mut store = PackageStore::new();

    let deps = vec![store.insert(compile::std())];
    let sources: [&str; 0] = [];
    let user = store.insert(compile(&store, [], sources, ""));

    ExecutionContextBuilder {
        store,
        package: user,
        compiler_builder: |store| Compiler::new(store, deps),
        globals_builder: extract_callables,
        env: None,
        cursor: Cursor::new(Vec::<u8>::new()),
        out_builder: |cursor| CursorReceiver::new(cursor),
    }
    .build()
}
