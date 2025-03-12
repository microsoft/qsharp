// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use rustc_hash::FxHashMap;

use super::types::Type;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub span: Span,
    pub ty: Type,
    pub qsharp_ty: crate::types::Type,
    pub io_kind: IOKind,
}

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
    id_to_symbol: FxHashMap<SymbolId, Symbol>,
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
    pub fn insert_symbol(&mut self, id: SymbolId, symbol: Symbol) -> Result<(), SymbolError> {
        if self.name_to_id.contains_key(&symbol.name) {
            return Err(SymbolError::AlreadyExists);
        }
        self.name_to_id.insert(symbol.name.clone(), id);
        self.id_to_symbol.insert(id, symbol);
        self.order.push(id);
        Ok(())
    }

    pub fn get_symbol_by_name(&self, name: &str) -> Option<(SymbolId, &Symbol)> {
        self.name_to_id
            .get(name)
            .and_then(|id| self.id_to_symbol.get(id).map(|s| (*id, s)))
    }

    fn get_ordered_symbols(&self) -> Vec<Symbol> {
        self.order
            .iter()
            .map(|id| self.id_to_symbol.get(id).expect("ID should exist").clone())
            .collect()
    }
}

/// A symbol table is a collection of scopes and manages the symbol ids.
pub struct SymbolTable {
    scopes: Vec<Scope>,
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

const BUILTIN_SYMBOLS: [&str; 6] = ["pi", "π", "tau", "τ", "euler", "ℇ"];

impl Default for SymbolTable {
    fn default() -> Self {
        let global = Scope::new(ScopeKind::Global);

        let mut slf = Self {
            scopes: vec![global],
            current_id: SymbolId::default(),
        };

        // Define global constants
        for symbol in BUILTIN_SYMBOLS {
            slf.insert_symbol(Symbol {
                name: symbol.to_string(),
                span: Span::default(),
                ty: Type::Float(None, true),
                qsharp_ty: crate::types::Type::Double(true),
                io_kind: IOKind::Default,
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
        let id = self.current_id;
        self.current_id = self.current_id.successor();
        self.scopes
            .last_mut()
            .expect("At least one scope should be available")
            .insert_symbol(id, symbol)?;

        Ok(id)
    }

    #[must_use]
    pub fn get_symbol_by_id(&self, id: SymbolId) -> Option<(SymbolId, &Symbol)> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.id_to_symbol.get(&id) {
                return Some((id, symbol));
            }
        }
        None
    }

    #[must_use]
    pub fn get_symbol_by_name(&self, name: &str) -> Option<(SymbolId, &Symbol)> {
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
            if let Some((id, symbol)) = scope.get_symbol_by_name(name) {
                return Some((id, symbol));
            }
        }

        if let Some(scope) = last_false {
            if let Some((id, symbol)) = scope.get_symbol_by_name(name) {
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
            if let Some((id, symbol)) = scope.get_symbol_by_name(name) {
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
    pub(crate) fn get_input(&self) -> Option<Vec<Symbol>> {
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
    pub(crate) fn get_output(&self) -> Option<Vec<Symbol>> {
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
    fn get_inferred_output(&self) -> Option<Vec<Symbol>> {
        let mut symbols = vec![];
        self.scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::Global)
            .for_each(|scope| {
                for symbol in scope
                    .get_ordered_symbols()
                    .into_iter()
                    .filter(|symbol| !BUILTIN_SYMBOLS.contains(&symbol.name.as_str()))
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
    fn get_io_output(&self) -> Option<Vec<Symbol>> {
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
    fn get_io_input(&self) -> Vec<Symbol> {
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
