// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::f64;
use std::rc::Rc;

use qsc_data_structures::{index_map::IndexMap, span::Span};
use rustc_hash::FxHashMap;

use super::{
    ast::{Expr, ExprKind, LiteralKind},
    types::Type,
};

/// We need a symbol table to keep track of the symbols in the program.
/// The scoping rules for QASM3 are slightly different from Q#. This also
/// means that we need to keep track of the input and output symbols in the
/// program. Additionally, we need to keep track of the types of the symbols
/// in the program for type checking.
/// Q# and QASM have different type systems, so we track the QASM semantic.
///
/// A symbol ID is a unique identifier for a symbol in the symbol table.
/// This is used to look up symbols in the symbol table.
/// Every symbol in the symbol table has a unique ID.
#[derive(Debug, Default, Clone, Copy)]
pub struct SymbolId(pub u32);

impl SymbolId {
    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u32> for SymbolId {
    fn from(val: u32) -> Self {
        SymbolId(val)
    }
}

impl From<SymbolId> for u32 {
    fn from(id: SymbolId) -> Self {
        id.0
    }
}

impl From<SymbolId> for usize {
    fn from(value: SymbolId) -> Self {
        value.0 as usize
    }
}

impl From<usize> for SymbolId {
    fn from(value: usize) -> Self {
        SymbolId(
            value
                .try_into()
                .unwrap_or_else(|_| panic!("Value, {value}, does not fit into SymbolId")),
        )
    }
}

impl PartialEq for SymbolId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SymbolId {}

impl PartialOrd for SymbolId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SymbolId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::hash::Hash for SymbolId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub span: Span,
    pub ty: Type,
    pub qsharp_ty: crate::types::Type,
    pub io_kind: IOKind,
    // Used for const evaluation.
    // This field should only be Some(_)
    // if the symbol is const.
    value: Option<Rc<Expr>>,
}

impl Symbol {
    #[must_use]
    pub fn new(
        name: &str,
        span: Span,
        ty: Type,
        qsharp_ty: crate::types::Type,
        io_kind: IOKind,
    ) -> Self {
        Self {
            name: name.to_string(),
            span,
            ty,
            qsharp_ty,
            io_kind,
            value: None,
        }
    }

    #[must_use]
    pub fn with_const_value(self, value: Rc<Expr>) -> Self {
        assert!(
            value.ty.is_const(),
            "this builder pattern should only be used with const expressions"
        );
        Symbol {
            value: Some(value),
            ..self
        }
    }

    /// Returns the value of the symbol.
    #[must_use]
    pub fn const_eval(&self) -> Rc<Expr> {
        if let Some(val) = &self.value {
            val.clone()
        } else {
            unreachable!("this function should only be called on const symbols");
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.span == other.span
            && self.ty == other.ty
            && self.qsharp_ty == other.qsharp_ty
            && self.io_kind == other.io_kind
    }
}

impl Eq for Symbol {}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::parser::ast::display_utils;
        display_utils::writeln_header(f, "Symbol", self.span)?;
        display_utils::writeln_field(f, "name", &self.name)?;
        display_utils::writeln_field(f, "type", &self.ty)?;
        display_utils::writeln_field(f, "qsharp_type", &self.qsharp_ty)?;
        display_utils::write_field(f, "io_kind", &self.io_kind)
    }
}

/// A symbol in the symbol table.
/// Default Q# type is Unit
impl Default for Symbol {
    fn default() -> Self {
        Self {
            name: String::default(),
            span: Span::default(),
            ty: Type::Err,
            qsharp_ty: crate::types::Type::Tuple(vec![]),
            io_kind: IOKind::default(),
            value: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    /// The symbol already exists in the symbol table, at the current scope.
    AlreadyExists,
}

/// Symbols have a an I/O kind that determines if they are input or output, or unspecified.
/// The default I/O kind means no explicit kind was part of the decl.
/// There is a specific statement for io decls which sets the I/O kind appropriately.
/// This is used to determine the input and output symbols in the program.
#[derive(Copy, Default, Debug, Clone, PartialEq, Eq)]
pub enum IOKind {
    #[default]
    Default,
    Input,
    Output,
}

impl std::fmt::Display for IOKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IOKind::Default => write!(f, "Default"),
            IOKind::Input => write!(f, "Input"),
            IOKind::Output => write!(f, "Output"),
        }
    }
}

/// A scope is a collection of symbols and a kind. The kind determines semantic
/// rules for compliation as shadowing and decl rules vary by scope kind.
pub(crate) struct Scope {
    /// A map from symbol name to symbol ID.
    name_to_id: FxHashMap<String, SymbolId>,
    /// A map from symbol ID to symbol.
    id_to_symbol: FxHashMap<SymbolId, Rc<Symbol>>,
    /// The order in which symbols were inserted into the scope.
    /// This is used to determine the order of symbols in the output.
    order: Vec<SymbolId>,
    /// The kind of the scope.
    kind: ScopeKind,
}

impl Scope {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            name_to_id: FxHashMap::default(),
            id_to_symbol: FxHashMap::default(),
            order: vec![],
            kind,
        }
    }

    /// Inserts the symbol into the current scope.
    /// Returns the ID of the symbol.
    ///
    /// # Errors
    ///
    /// This function will return an error if a symbol of the same name has already
    /// been declared in this scope.
    pub fn insert_symbol(&mut self, id: SymbolId, symbol: Rc<Symbol>) -> Result<(), SymbolError> {
        if self.name_to_id.contains_key(&symbol.name) {
            return Err(SymbolError::AlreadyExists);
        }
        self.name_to_id.insert(symbol.name.clone(), id);
        self.id_to_symbol.insert(id, symbol);
        self.order.push(id);
        Ok(())
    }

    pub fn get_symbol_by_name(&self, name: &str) -> Option<(SymbolId, Rc<Symbol>)> {
        self.name_to_id
            .get(name)
            .and_then(|id| self.id_to_symbol.get(id).map(|s| (*id, s.clone())))
    }

    fn get_ordered_symbols(&self) -> Vec<Rc<Symbol>> {
        self.order
            .iter()
            .map(|id| self.id_to_symbol.get(id).expect("ID should exist").clone())
            .collect()
    }
}

/// A symbol table is a collection of scopes and manages the symbol ids.
pub struct SymbolTable {
    scopes: Vec<Scope>,
    symbols: IndexMap<SymbolId, Rc<Symbol>>,
    current_id: SymbolId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    /// Global scope, which is the current scope only when no other scopes are active.
    /// This is the only scope where gates, qubits, and arrays can be declared.
    Global,
    Function,
    Gate,
    Block,
}

const BUILTIN_SYMBOLS: [(&str, f64); 6] = [
    ("pi", f64::consts::PI),
    ("π", f64::consts::PI),
    ("tau", f64::consts::TAU),
    ("τ", f64::consts::TAU),
    ("euler", f64::consts::E),
    ("ℇ", f64::consts::E),
];

impl Default for SymbolTable {
    fn default() -> Self {
        let global = Scope::new(ScopeKind::Global);

        let mut slf = Self {
            scopes: vec![global],
            symbols: IndexMap::default(),
            current_id: SymbolId::default(),
        };

        // Define global constants.
        for (symbol, val) in BUILTIN_SYMBOLS {
            let ty = Type::Float(None, true);
            let expr = Expr {
                span: Span::default(),
                kind: Box::new(ExprKind::Lit(LiteralKind::Float(val))),
                ty: ty.clone(),
            };

            slf.insert_symbol(Symbol {
                name: symbol.to_string(),
                span: Span::default(),
                ty,
                qsharp_ty: crate::types::Type::Double(true),
                io_kind: IOKind::Default,
                value: Some(Rc::new(expr)),
            })
            .unwrap_or_else(|_| panic!("Failed to insert symbol: {symbol}"));
        }

        slf
    }
}

impl SymbolTable {
    pub fn push_scope(&mut self, kind: ScopeKind) {
        assert!(kind != ScopeKind::Global, "Cannot push a global scope");
        self.scopes.push(Scope::new(kind));
    }

    pub fn pop_scope(&mut self) {
        assert!(self.scopes.len() != 1, "Cannot pop the global scope");
        self.scopes.pop();
    }

    pub fn insert_symbol(&mut self, symbol: Symbol) -> Result<SymbolId, SymbolError> {
        let symbol = Rc::new(symbol);
        let id = self.current_id;
        match self
            .scopes
            .last_mut()
            .expect("At least one scope should be available")
            .insert_symbol(id, symbol.clone())
        {
            Ok(()) => {
                self.current_id = self.current_id.successor();
                self.symbols.insert(id, symbol);
                Ok(id)
            }
            Err(SymbolError::AlreadyExists) => Err(SymbolError::AlreadyExists),
        }
    }

    fn insert_err_symbol(&mut self, name: &str, span: Span) -> (SymbolId, Rc<Symbol>) {
        let symbol = Rc::new(Symbol {
            name: name.to_string(),
            span,
            ty: Type::Err,
            qsharp_ty: crate::types::Type::Err,
            io_kind: IOKind::Default,
            value: None,
        });
        let id = self.current_id;
        self.current_id = self.current_id.successor();
        self.symbols.insert(id, symbol.clone());
        (id, symbol)
    }

    /// Gets the symbol with the given ID, or creates it with the given name and span.
    /// the boolean value indicates if the symbol was created or not.
    pub fn try_get_existing_or_insert_err_symbol(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<(SymbolId, Rc<Symbol>), (SymbolId, Rc<Symbol>)> {
        // if we have the symbol, return it, otherswise create it with err values
        if let Some((id, symbol)) = self.get_symbol_by_name(name) {
            return Ok((id, symbol.clone()));
        }
        // if we don't have the symbol, create it with err values
        Err(self.insert_err_symbol(name, span))
    }

    pub fn try_insert_or_get_existing(&mut self, symbol: Symbol) -> Result<SymbolId, SymbolId> {
        let name = symbol.name.clone();
        if let Ok(symbol_id) = self.insert_symbol(symbol) {
            Ok(symbol_id)
        } else {
            let symbol_id = self
                .get_symbol_by_name(&name)
                .map(|(id, _)| id)
                .expect("msg");
            Err(symbol_id)
        }
    }

    /// Gets the symbol with the given name. This should only be used if you don't
    /// have the symbold ID. This function will search the scopes in reverse order
    /// and return the first symbol with the given name following the scoping rules.
    #[must_use]
    pub fn get_symbol_by_name<S>(&self, name: S) -> Option<(SymbolId, Rc<Symbol>)>
    where
        S: AsRef<str>,
    {
        let scopes = self.scopes.iter().rev();
        let predicate = |x: &Scope| {
            x.kind == ScopeKind::Block || x.kind == ScopeKind::Function || x.kind == ScopeKind::Gate
        };

        // Use scan to track the last item that returned false
        let mut last_false = None;
        let _ = scopes
            .scan(&mut last_false, |state, item| {
                if !predicate(item) {
                    **state = Some(item);
                }
                Some(predicate(item))
            })
            .take_while(|&x| x)
            .last();
        let mut scopes = self.scopes.iter().rev();
        while let Some(scope) = scopes
            .by_ref()
            .take_while(|arg0: &&Scope| predicate(arg0))
            .next()
        {
            if let Some((id, symbol)) = scope.get_symbol_by_name(name.as_ref()) {
                return Some((id, symbol));
            }
        }

        if let Some(scope) = last_false {
            if let Some((id, symbol)) = scope.get_symbol_by_name(name.as_ref()) {
                if symbol.ty.is_const()
                    || matches!(symbol.ty, Type::Gate(..) | Type::Void)
                    || self.is_scope_rooted_in_global()
                {
                    return Some((id, symbol));
                }
            }
        }
        // we should be at the global, function, or gate scope now
        for scope in scopes {
            if let Some((id, symbol)) = scope.get_symbol_by_name(name.as_ref()) {
                if symbol.ty.is_const() || matches!(symbol.ty, Type::Gate(..) | Type::Void) {
                    return Some((id, symbol));
                }
            }
        }

        None
    }

    #[must_use]
    pub fn is_current_scope_global(&self) -> bool {
        matches!(self.scopes.last(), Some(scope) if scope.kind == ScopeKind::Global)
    }

    #[must_use]
    pub fn is_scope_rooted_in_subroutine(&self) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.kind == ScopeKind::Function)
    }

    #[must_use]
    pub fn is_scope_rooted_in_global(&self) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.kind == ScopeKind::Function {
                return false;
            }
            if scope.kind == ScopeKind::Gate {
                return false;
            }
        }
        true
    }

    /// Get the input symbols in the program.
    pub(crate) fn get_input(&self) -> Option<Vec<Rc<Symbol>>> {
        let io_input = self.get_io_input();
        if io_input.is_empty() {
            None
        } else {
            Some(io_input)
        }
    }

    /// Get the output symbols in the program.
    /// Output symbols are either inferred or explicitly declared.
    /// If there are no explicitly declared output symbols, then the inferred
    /// output symbols are returned.
    pub(crate) fn get_output(&self) -> Option<Vec<Rc<Symbol>>> {
        let io_ouput = self.get_io_output();
        if io_ouput.is_some() {
            io_ouput
        } else {
            self.get_inferred_output()
        }
    }

    /// Get all symbols in the global scope that are inferred output symbols.
    /// Any global symbol that is not a built-in symbol and has a type that is
    /// inferred to be an output type is considered an inferred output symbol.
    fn get_inferred_output(&self) -> Option<Vec<Rc<Symbol>>> {
        let mut symbols = vec![];
        self.scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::Global)
            .for_each(|scope| {
                for symbol in scope
                    .get_ordered_symbols()
                    .into_iter()
                    .filter(|symbol| {
                        !BUILTIN_SYMBOLS
                            .map(|pair| pair.0)
                            .contains(&symbol.name.as_str())
                    })
                    .filter(|symbol| symbol.io_kind == IOKind::Default)
                {
                    if symbol.ty.is_inferred_output_type() {
                        symbols.push(symbol);
                    }
                }
            });
        if symbols.is_empty() {
            None
        } else {
            Some(symbols)
        }
    }

    /// Get all symbols in the global scope that are output symbols.
    fn get_io_output(&self) -> Option<Vec<Rc<Symbol>>> {
        let mut symbols = vec![];
        for scope in self
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::Global)
        {
            for symbol in scope.get_ordered_symbols() {
                if symbol.io_kind == IOKind::Output {
                    symbols.push(symbol);
                }
            }
        }
        if symbols.is_empty() {
            None
        } else {
            Some(symbols)
        }
    }

    /// Get all symbols in the global scope that are input symbols.
    fn get_io_input(&self) -> Vec<Rc<Symbol>> {
        let mut symbols = vec![];
        for scope in self
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::Global)
        {
            for symbol in scope.get_ordered_symbols() {
                if symbol.io_kind == IOKind::Input {
                    symbols.push(symbol);
                }
            }
        }
        symbols
    }
}

impl std::ops::Index<SymbolId> for SymbolTable {
    type Output = Rc<Symbol>;

    fn index(&self, index: SymbolId) -> &Self::Output {
        self.symbols.get(index).expect("Symbol should exist")
    }
}
