// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::*;
use std::io::prelude::*;
use std::io::LineWriter;

use qsc_ast::visit::Visitor;

#[cfg(test)]
mod tests;

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

    fn print_bin_op(&mut self, op: BinOp) {
        match op {
            BinOp::Add => self.print("+"),
            BinOp::AndB => self.print("&&&"),
            BinOp::AndL => self.print("and"),
            BinOp::Div => self.print("/"),
            BinOp::Eq => self.print("=="),
            BinOp::Exp => self.print("^"),
            BinOp::Gt => self.print(">"),
            BinOp::Gte => self.print(">="),
            BinOp::Lt => self.print("<"),
            BinOp::Lte => self.print("<="),
            BinOp::Mod => self.print("%"),
            BinOp::Mul => self.print("*"),
            BinOp::Neq => self.print("!="),
            BinOp::OrB => self.print("|||"),
            BinOp::OrL => self.print("or"),
            BinOp::Shl => self.print("<<<"),
            BinOp::Shr => self.print(">>>"),
            BinOp::Sub => self.print("-"),
            BinOp::XorB => self.print("^^^"),
        }
    }

    fn print_un_op(&mut self, op: UnOp) {
        match op {
            UnOp::Functor(f) => match f {
                Functor::Adj => self.print("Adjoint "),
                Functor::Ctl => self.print("Controlled "),
            },
            UnOp::Neg => self.print("-"),
            UnOp::NotB => self.print("~~~"),
            UnOp::NotL => self.print("not "),
            UnOp::Pos => self.print("+"),
            UnOp::Unwrap => self.print("!"),
        }
    }

    fn print_lit(&mut self, lit: &Lit) {
        match lit {
            Lit::BigInt(bi) => self.print(&bi.to_string()),
            Lit::Bool(b) => {
                if *b {
                    self.print("true");
                } else {
                    self.print("false");
                }
            }
            Lit::Double(d) => self.print(&d.to_string()),
            Lit::Int(i) => self.print(&i.to_string()),
            Lit::Pauli(pauli) => match pauli {
                Pauli::I => self.print("PauliI"),
                Pauli::X => self.print("PauliX"),
                Pauli::Y => self.print("PauliY"),
                Pauli::Z => self.print("PauliZ"),
            },
            Lit::Result(result) => match result {
                Result::Zero => self.print("Zero"),
                Result::One => self.print("One"),
            },
            Lit::String(s) => self.print(&format!("\"{s}\"")),
        }
    }

    fn visit_array(&mut self, exprs: &Vec<Expr>) {
        self.print("[");
        exprs.iter().fold(true, |first, e| {
            if !first {
                self.print(", ");
            }
            self.visit_expr(e);
            false
        });
        self.print("]");
    }

    fn visit_array_repeat(&mut self, item: &Expr, size: &Expr) {
        self.print("[");
        self.visit_expr(item);
        self.print(", size = ");
        self.visit_expr(size);
        self.print("]");
    }

    fn visit_assign(&mut self, lhs: &Expr, rhs: &Expr) {
        self.print("set ");
        self.visit_expr(lhs);
        self.print(" = ");
        self.visit_expr(rhs);
    }

    fn visit_assign_op(&mut self, op: &BinOp, lhs: &Expr, rhs: &Expr) {
        self.print("set ");
        self.visit_expr(lhs);
        self.print(" ");
        self.print_bin_op(*op);
        self.print("= ");
        self.visit_expr(rhs);
    }

    fn visit_assign_update(&mut self, record: &Expr, index: &Expr, value: &Expr) {
        self.print("set ");
        self.visit_expr(record);
        self.print(" w/= ");
        self.visit_expr(index);
        self.print(" <- ");
        self.visit_expr(value);
    }

    fn visit_bin_op(&mut self, op: &BinOp, lhs: &Expr, rhs: &Expr) {
        self.visit_expr(lhs);
        self.print(" ");
        self.print_bin_op(*op);
        self.print(" ");
        self.visit_expr(rhs);
    }

    fn visit_call(&mut self, callee: &Expr, arg: &Expr) {
        self.visit_expr(callee);
        self.visit_expr(arg);
    }

    fn visit_conjugate(&mut self, within: &Block, apply: &Block) {
        self.print("within ");
        self.visit_block(within);
        self.print(" apply ");
        self.visit_block(apply);
    }

    fn visit_err(&mut self) {}

    fn visit_fail(&mut self, msg: &Expr) {
        self.print("fail ");
        self.visit_expr(msg);
    }

    fn visit_field(&mut self, record: &Expr, name: &Ident) {
        self.visit_expr(record);
        self.print("::");
        self.visit_ident(name);
    }

    fn visit_for(&mut self, pat: &Pat, iter: &Expr, block: &Block) {
        self.print("for ");
        self.visit_pat(pat);
        self.print(" in ");
        self.visit_expr(iter);
        self.print(" ");
        self.visit_block(block);
    }

    fn visit_hole(&mut self) {
        self.print("_");
    }

    fn visit_if(&mut self, cond: &Expr, body: &Block, otherwise: &Option<Box<Expr>>) {
        self.print("if ");
        self.visit_expr(cond);
        self.print(" ");
        self.visit_block(body);
        if let Some(e) = otherwise {
            self.print(" else ");
            self.visit_expr(e);
        }
    }

    fn visit_index(&mut self, array: &Expr, index: &Expr) {
        self.visit_expr(array);
        self.print("[");
        self.visit_expr(index);
        self.print("]");
    }

    fn visit_lambda(&mut self, kind: &CallableKind, pat: &Pat, expr: &Expr) {
        self.visit_pat(pat);
        match kind {
            CallableKind::Function => self.print(" -> "),
            CallableKind::Operation => self.print(" => "),
        }
        self.visit_expr(expr);
    }

    fn visit_lit(&mut self, lit: &Lit) {
        self.print_lit(lit);
    }

    fn visit_paren(&mut self, expr: &Expr) {
        self.print("(");
        self.visit_expr(expr);
        self.print(")");
    }

    fn visit_range(
        &mut self,
        start: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) {
        if start.is_none() && step.is_none() && end.is_none() {
            self.print("...");
        } else {
            match start {
                Some(s) => self.visit_expr(s),
                None => self.print("."),
            }

            self.print("..");

            if let Some(m) = step {
                self.visit_expr(m);
                self.print("..");
            }

            match end {
                Some(e) => self.visit_expr(e),
                None => self.print("."),
            }
        }
    }

    fn visit_repeat(&mut self, body: &Block, until: &Expr, fixup: &Option<Block>) {
        self.print("repeat ");
        self.visit_block(body);
        self.print("\n");
        self.print_tabs();
        self.print("until ");
        self.visit_expr(until);
        if let Some(f) = fixup {
            self.print("\n");
            self.print_tabs();
            self.print("fixup ");
            self.visit_block(f);
        }
    }

    fn visit_return(&mut self, expr: &Expr) {
        self.print("return ");
        self.visit_expr(expr);
    }

    fn visit_tern_op(&mut self, op: &TernOp, e1: &Expr, e2: &Expr, e3: &Expr) {
        self.visit_expr(e1);
        match op {
            TernOp::Cond => {
                self.print(" ? ");
                self.visit_expr(e2);
                self.print(" | ");
            }
            TernOp::Update => {
                self.print(" w/ ");
                self.visit_expr(e2);
                self.print(" <- ");
            }
        }
        self.visit_expr(e3);
    }

    fn visit_tuple(&mut self, exprs: &Vec<Expr>) {
        self.print("(");
        exprs.iter().fold(true, |first, e| {
            if !first {
                self.print(", ");
            }
            self.visit_expr(e);
            false
        });
        self.print(")");
    }

    fn visit_un_op(&mut self, op: &UnOp, expr: &Expr) {
        self.print_un_op(*op);
        self.visit_expr(expr);
    }

    fn visit_while(&mut self, cond: &Expr, block: &Block) {
        self.print("while ");
        self.visit_expr(cond);
        self.print(" ");
        self.visit_block(block);
    }
}

impl<'a, W> Visitor<'a> for CodePrinter<W>
where
    W: Write,
{
    fn visit_package(&mut self, package: &'a Package) {
        package.namespaces.iter().fold(true, |first, n| {
            if !first {
                self.print("\n");
            }
            self.visit_namespace(n);
            false
        });
    }

    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.print("namespace ");
        self.visit_ident(&namespace.name);
        self.print(" {\n");
        self.indentation += 1;
        for i in &namespace.items {
            self.visit_item(i);
        }
        self.indentation -= 1;
        self.print_tabs();
        self.print("}\n");
    }

    fn visit_item(&mut self, item: &'a Item) {
        match &item.kind {
            ItemKind::Callable(decl) => {
                self.print("\n");
                self.print_tabs();
                item.meta.attrs.iter().for_each(|a| self.visit_attr(a));
                self.visit_callable_decl(decl);
            }
            ItemKind::Err => {}
            ItemKind::Open(ns, alias) => {
                self.print_tabs();
                self.print("open ");
                self.visit_ident(ns);
                if let Some(a) = alias {
                    self.print(" as ");
                    self.visit_ident(a);
                }
                self.print(";\n");
            }
            ItemKind::Ty(ident, def) => {
                self.print("\n");
                self.print_tabs();
                item.meta.attrs.iter().for_each(|a| self.visit_attr(a));
                self.print("newtype ");
                self.visit_ident(ident);
                self.print(" = ");
                self.visit_ty_def(def);
                self.print(";\n");
            }
        }
    }

    fn visit_attr(&mut self, attr: &'a Attr) {
        self.print("@ ");
        self.visit_path(&attr.name);
        self.visit_expr(&attr.arg);
        self.print("\n");
        self.print_tabs();
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
                defs.iter().fold(true, |first, d| {
                    if !first {
                        self.print(", ");
                    }
                    self.visit_ty_def(d);
                    false
                });
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
            decl.ty_params.iter().fold(true, |first, ty| {
                if !first {
                    self.print(", ");
                }
                self.print("'");
                self.visit_ident(ty);
                false
            });
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

                specs.iter().fold(true, |first, s| {
                    if !first {
                        self.print("\n");
                    }
                    self.print_tabs();
                    self.visit_spec_decl(s);
                    false
                });

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
            TyKind::App(ty, tys) => {
                let mut is_array = false;

                if let TyKind::Prim(p) = ty.kind {
                    if TyPrim::Array == p {
                        is_array = true;
                        let t = tys.first().expect("multiple types for array type");
                        self.visit_ty(t);
                        self.print("[]");
                    }
                }

                if !is_array {
                    self.visit_ty(ty);
                    if !tys.is_empty() {
                        self.print("<");
                        tys.iter().fold(true, |first, t| {
                            if !first {
                                self.print(", ");
                            }
                            self.visit_ty(t);
                            false
                        });
                        self.print(">");
                    }
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
                    tys.iter().fold(true, |first, t| {
                        if !first {
                            self.print(", ");
                        }
                        self.visit_ty(t);
                        false
                    });
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
            StmtKind::Expr(expr) => self.visit_expr(expr),
            StmtKind::Semi(expr) => {
                self.visit_expr(expr);
                self.print(";");
            }
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
            ExprKind::Array(exprs) => self.visit_array(exprs),
            ExprKind::ArrayRepeat(item, size) => self.visit_array_repeat(item, size),
            ExprKind::Assign(lhs, rhs) => self.visit_assign(lhs, rhs),
            ExprKind::AssignOp(op, lhs, rhs) => self.visit_assign_op(op, lhs, rhs),
            ExprKind::AssignUpdate(record, index, value) => {
                self.visit_assign_update(record, index, value);
            }
            ExprKind::BinOp(op, lhs, rhs) => self.visit_bin_op(op, lhs, rhs),
            ExprKind::Block(block) => self.visit_block(block),
            ExprKind::Call(callee, arg) => self.visit_call(callee, arg),
            ExprKind::Conjugate(within, apply) => self.visit_conjugate(within, apply),
            ExprKind::Err => self.visit_err(),
            ExprKind::Fail(msg) => self.visit_fail(msg),
            ExprKind::Field(record, name) => self.visit_field(record, name),
            ExprKind::For(pat, iter, block) => self.visit_for(pat, iter, block),
            ExprKind::Hole => self.visit_hole(),
            ExprKind::If(cond, body, otherwise) => self.visit_if(cond, body, otherwise),
            ExprKind::Index(array, index) => self.visit_index(array, index),
            ExprKind::Lambda(kind, pat, expr) => self.visit_lambda(kind, pat, expr),
            ExprKind::Lit(lit) => self.visit_lit(lit),
            ExprKind::Paren(expr) => self.visit_paren(expr),
            ExprKind::Path(path) => self.visit_path(path),
            ExprKind::Range(start, step, end) => self.visit_range(start, step, end),
            ExprKind::Repeat(body, until, fixup) => self.visit_repeat(body, until, fixup),
            ExprKind::Return(expr) => self.visit_return(expr),
            ExprKind::TernOp(op, e1, e2, e3) => self.visit_tern_op(op, e1, e2, e3),
            ExprKind::Tuple(exprs) => self.visit_tuple(exprs),
            ExprKind::UnOp(op, expr) => self.visit_un_op(op, expr),
            ExprKind::While(cond, block) => self.visit_while(cond, block),
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
                pats.iter().fold(true, |first, p| {
                    if !first {
                        self.print(", ");
                    }
                    self.visit_pat(p);
                    false
                });
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
                inits.iter().fold(true, |first, i| {
                    if !first {
                        self.print(", ");
                    }
                    self.visit_qubit_init(i);
                    false
                });
                self.print(")");
            }
        }
    }

    fn visit_ident(&mut self, id: &'a Ident) {
        self.print(&id.name);
    }
}
