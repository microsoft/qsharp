// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::*;
use std::io::prelude::*;
use std::io::LineWriter;

use qsc_ast::visit::Visitor;

pub struct CodePrinter<W: Write> {
    pub writer: LineWriter<W>,
    pub indentation: i32,
}

impl<W> CodePrinter<W>
where
    W: Write,
{
    fn print(&mut self, s: &str) {
        self.writer
            .write_all(s.as_bytes())
            .expect("printing didn't work");
    }

    fn print_tabs(&mut self) {
        for _ in 0..self.indentation {
            self.print("    ");
        }
    }

    fn print_prim_type(&mut self, t: TyPrim) {
        match t {
            TyPrim::Array => self.print("[]"),
            TyPrim::BigInt => self.print("BigInt"),
            TyPrim::Bool => self.print("Bool"),
            TyPrim::Double => self.print("Double"),
            TyPrim::Int => self.print("Int"),
            TyPrim::Pauli => self.print("Pauli"),
            TyPrim::Qubit => self.print("Qubit"),
            TyPrim::Range => self.print("Range"),
            TyPrim::Result => self.print("Result"),
            TyPrim::String => self.print("String"),
        }
    }
}

impl<'a, W> Visitor<'a> for CodePrinter<W>
where
    W: Write,
{
    fn visit_package(&mut self, package: &'a Package) {
        let mut is_first = true;
        for n in &package.namespaces {
            if !is_first {
                self.print("\n");
            }
            self.visit_namespace(n);
            is_first = false;
        }
    }

    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.print("namespace ");
        self.visit_ident(&namespace.name);
        self.print(" {\n");
        self.indentation += 1;
        for i in &namespace.items {
            self.print("\n");
            self.print_tabs();
            self.visit_item(i);
        }
        self.indentation -= 1;
        self.print_tabs();
        self.print("}\n");
    }

    fn visit_item(&mut self, item: &'a Item) {
        item.meta.attrs.iter().for_each(|a| self.visit_attr(a));
        match &item.kind {
            ItemKind::Callable(decl) => self.visit_callable_decl(decl),
            ItemKind::Err => {}
            ItemKind::Open(ns, alias) => {
                self.print("open ");
                self.visit_ident(ns);
                if let Some(a) = alias {
                    self.print(" as ");
                    self.visit_ident(a);
                }
                self.print(";");
            }
            ItemKind::Ty(ident, def) => {
                self.print("newtype ");
                self.visit_ident(ident);
                self.print(" = ");
                self.visit_ty_def(def);
                self.print(";");
            }
        }
    }

    fn visit_attr(&mut self, attr: &'a Attr) {
        self.print("@ ");
        self.visit_path(&attr.name);
        self.visit_expr(&attr.arg);
    }

    fn visit_ty_def(&mut self, def: &'a TyDef) {
        match &def.kind {
            TyDefKind::Field(name, ty) => {
                if let Some(n) = name {
                    self.visit_ident(n);
                    self.print(": ");
                }
                self.visit_ty(ty);
            }
            TyDefKind::Paren(def) => {
                self.print("(");
                self.visit_ty_def(def);
                self.print(")");
            }
            TyDefKind::Tuple(defs) => {
                self.print("(");
                let mut is_first = true;
                for d in defs {
                    if !is_first {
                        self.print(", ");
                    }
                    self.visit_ty_def(d);
                    is_first = false;
                }
                self.print(")");
            }
        }
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        match decl.kind {
            CallableKind::Function => self.print("function "),
            CallableKind::Operation => self.print("operation "),
        }
        self.visit_ident(&decl.name);
        if !decl.ty_params.is_empty() {
            self.print("<");
            let mut is_first = true;
            for ty in &decl.ty_params {
                if !is_first {
                    self.print(", ");
                }
                self.print("'");
                self.visit_ident(ty);
                is_first = false;
            }
            self.print(">");
        }
        self.print(" ");
        self.visit_pat(&decl.input);
        self.print(" : ");
        self.visit_ty(&decl.output);
        self.print(" ");
        if let Some(f) = &decl.functors {
            self.print("is ");
            self.visit_functor_expr(f);
            self.print(" ");
        }
        match &decl.body {
            CallableBody::Block(block) => self.visit_block(block),
            CallableBody::Specs(specs) => {
                self.print("{\n");
                self.indentation += 1;

                for s in specs {
                    self.print("\n");
                    self.print_tabs();
                    self.visit_spec_decl(s);
                }

                self.indentation -= 1;
                self.print_tabs();
                self.print("}");
            }
        }
        self.print("\n");
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        match decl.spec {
            Spec::Body => self.print("body"),
            Spec::Adj => self.print("adjoint"),
            Spec::Ctl => self.print("controlled"),
            Spec::CtlAdj => self.print("controlled adjoint"),
        }

        match &decl.body {
            SpecBody::Gen(gen) => match gen {
                SpecGen::Auto => self.print(" auto;"),
                SpecGen::Distribute => self.print(" distribute;"),
                SpecGen::Intrinsic => self.print(" intrinsic;"),
                SpecGen::Invert => self.print(" invert;"),
                SpecGen::Slf => self.print(" self;"),
            },
            SpecBody::Impl(pat, block) => {
                self.visit_pat(pat);
                self.print(" ");
                self.visit_block(block);
            }
        }
        self.print("\n");
    }

    fn visit_functor_expr(&mut self, expr: &'a FunctorExpr) {
        match &expr.kind {
            FunctorExprKind::BinOp(op, lhs, rhs) => {
                self.visit_functor_expr(lhs);
                match op {
                    SetOp::Union => self.print(" + "),
                    SetOp::Intersect => self.print(" * "),
                }
                self.visit_functor_expr(rhs);
            }
            FunctorExprKind::Lit(fun) => match fun {
                Functor::Adj => self.print("Adj"),
                Functor::Ctl => self.print("Ctl"),
            },
            FunctorExprKind::Paren(expr) => {
                self.print("(");
                self.visit_functor_expr(expr);
                self.print(")");
            }
        }
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        match &ty.kind {
            // ToDo: this doesn't handle arrays well.
            TyKind::App(ty, tys) => {
                self.visit_ty(ty);
                if !tys.is_empty() {
                    self.print("<");
                    let mut is_first = true;
                    for t in tys {
                        if !is_first {
                            self.print(", ");
                        }
                        self.visit_ty(t);
                        is_first = false;
                    }
                    self.print(">");
                }
            }
            TyKind::Arrow(kind, lhs, rhs, functors) => {
                self.visit_ty(lhs);
                match kind {
                    CallableKind::Function => self.print(" -> "),
                    CallableKind::Operation => self.print(" => "),
                }
                self.visit_ty(rhs);
                if let Some(f) = functors {
                    self.print(" is ");
                    self.visit_functor_expr(f);
                }
            }
            TyKind::Hole => self.print("_"),
            TyKind::Paren(ty) => {
                self.print("(");
                self.visit_ty(ty);
                self.print(")");
            }
            TyKind::Path(path) => self.visit_path(path),
            TyKind::Prim(prim) => self.print_prim_type(*prim),
            TyKind::Tuple(tys) => {
                if tys.is_empty() {
                    self.print("Unit");
                } else {
                    self.print("(");
                    let mut is_first = true;
                    for t in tys {
                        if !is_first {
                            self.print(", ");
                        }
                        self.visit_ty(t);
                        is_first = false;
                    }
                    self.print(")");
                }
            }
            TyKind::Var(tvar) => {
                self.print("'");
                match tvar {
                    TyVar::Name(n) => self.print(n),
                    TyVar::Id(i) => self.print(&i.to_string()),
                }
            }
        }
    }

    fn visit_block(&mut self, block: &'a Block) {
        self.print("{\n");
        self.indentation += 1;
        for s in &block.stmts {
            self.print_tabs();
            self.visit_stmt(s);
        }
        self.indentation -= 1;
        self.print_tabs();
        self.print("}");
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match &stmt.kind {
            StmtKind::Empty => {}
            StmtKind::Expr(expr) | StmtKind::Semi(expr) => self.visit_expr(expr),
            StmtKind::Local(m, pat, value) => {
                match m {
                    Mutability::Immutable => self.print("let "),
                    Mutability::Mutable => self.print("mutable "),
                }
                self.visit_pat(pat);
                self.print(" = ");
                self.visit_expr(value);
                self.print(";");
            }
            StmtKind::Qubit(u, pat, init, block) => {
                match u {
                    QubitSource::Dirty => self.print("borrow "),
                    QubitSource::Fresh => self.print("use "),
                }
                self.visit_pat(pat);
                self.print(" = ");
                self.visit_qubit_init(init);
                match &block {
                    Some(b) => {
                        self.print(" ");
                        self.visit_block(b);
                    }
                    None => self.print(";"),
                }
            }
        }
        self.print("\n");
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::Array(_) => todo!(),
            ExprKind::ArrayRepeat(_, _) => todo!(),
            ExprKind::Assign(_, _) => todo!(),
            ExprKind::AssignOp(_, _, _) => todo!(),
            ExprKind::AssignUpdate(_, _, _) => todo!(),
            ExprKind::BinOp(_, _, _) => todo!(),
            ExprKind::Block(_) => todo!(),
            ExprKind::Call(_, _) => todo!(),
            ExprKind::Conjugate(_, _) => todo!(),
            ExprKind::Err => todo!(),
            ExprKind::Fail(_) => todo!(),
            ExprKind::Field(_, _) => todo!(),
            ExprKind::For(_, _, _) => todo!(),
            ExprKind::Hole => todo!(),
            ExprKind::If(_, _, _) => todo!(),
            ExprKind::Index(_, _) => todo!(),
            ExprKind::Lambda(_, _, _) => todo!(),
            ExprKind::Lit(lit) => todo!(),
            ExprKind::Paren(_) => todo!(),
            ExprKind::Path(_) => todo!(),
            ExprKind::Range(_, _, _) => todo!(),
            ExprKind::Repeat(_, _, _) => todo!(),
            ExprKind::Return(_) => todo!(),
            ExprKind::TernOp(_, _, _, _) => todo!(),
            ExprKind::Tuple(_) => todo!(),
            ExprKind::UnOp(_, _) => todo!(),
            ExprKind::While(_, _) => todo!(),
        }
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        match &pat.kind {
            PatKind::Bind(name, ty) => {
                self.visit_ident(name);
                if let Some(t) = ty {
                    self.print(": ");
                    self.visit_ty(t);
                }
            }
            PatKind::Discard(ty) => {
                self.print("_");
                if let Some(t) = ty {
                    self.print(": ");
                    self.visit_ty(t);
                }
            }
            PatKind::Elided => self.print("..."),
            PatKind::Paren(pat) => {
                self.print("(");
                self.visit_pat(pat);
                self.print(")");
            }
            PatKind::Tuple(pats) => {
                self.print("(");
                let mut is_first = true;
                for p in pats {
                    if !is_first {
                        self.print(", ");
                    }
                    self.visit_pat(p);
                    is_first = false;
                }
                self.print(")");
            }
        }
    }

    fn visit_path(&mut self, path: &'a Path) {
        if let Some(n) = &path.namespace {
            self.visit_ident(n);
            self.print(".");
        }
        self.visit_ident(&path.name);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        match &init.kind {
            QubitInitKind::Array(len) => {
                self.print("Qubit[");
                self.visit_expr(len);
                self.print("]");
            }
            QubitInitKind::Paren(init) => {
                self.print("(");
                self.visit_qubit_init(init);
                self.print(")");
            }
            QubitInitKind::Single => self.print("Qubit()"),
            QubitInitKind::Tuple(inits) => {
                self.print("(");
                let mut is_first = true;
                for i in inits {
                    if !is_first {
                        self.print(", ");
                    }
                    self.visit_qubit_init(i);
                    is_first = false;
                }
                self.print(")");
            }
        }
    }

    fn visit_ident(&mut self, id: &'a Ident) {
        self.print(&id.name);
    }
}
