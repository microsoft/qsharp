// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use oq3_semantics::types::{IsConst, Type};
use qsc::Span;
use rustc_hash::FxHashMap;

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
            value.try_into().unwrap_or_else(|_| {
                panic!("Value, {value}, does not fit into {}", stringify!($id))
            }),
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
pub(crate) struct Symbol {
    pub name: String,
    pub span: Span,
    pub ty: Type,
    pub qsharp_ty: crate::types::Type,
    pub io_kind: IOKind,
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            name: String::default(),
            span: Span::default(),
            ty: Type::Undefined,
            qsharp_ty: crate::types::Type::Tuple(vec![]),
            io_kind: IOKind::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    NotFound,
    AlreadyExists,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IOKind {
    Default,
    Input,
    Output,
}

impl Default for IOKind {
    fn default() -> Self {
        Self::Default
    }
}
pub struct Scope {
    name_to_id: FxHashMap<String, SymbolId>,
    id_to_symbol: FxHashMap<SymbolId, Symbol>,
    order: Vec<SymbolId>,
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

    pub fn insert_symbol(&mut self, id: SymbolId, symbol: Symbol) -> Result<(), SymbolError> {
        if self.name_to_id.contains_key(&symbol.name) {
            return Err(SymbolError::AlreadyExists);
        }
        self.name_to_id.insert(symbol.name.clone(), id);
        self.id_to_symbol.insert(id, symbol);
        self.order.push(id);
        Ok(())
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.name_to_id
            .get(name)
            .and_then(|id| self.id_to_symbol.get(id))
    }

    pub fn get_symbol_by_id(&self, id: SymbolId) -> Option<&Symbol> {
        self.id_to_symbol.get(&id)
    }

    fn get_ordered_symbols(&self) -> Vec<Symbol> {
        self.order
            .iter()
            .map(|id| self.id_to_symbol.get(id).expect("ID should exist").clone())
            .collect()
    }
}

pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_id: SymbolId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Function,
    Gate,
    Block,
}

const BUILTIN_SYMBOLS: [&str; 6] = ["pi", "π", "tau", "τ", "euler", "ℇ"];

impl SymbolTable {
    pub fn new() -> Self {
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
                ty: Type::Float(None, IsConst::True),
                qsharp_ty: crate::types::Type::Double(true),
                io_kind: IOKind::Default,
            })
            .unwrap_or_else(|_| panic!("Failed to insert symbol: {symbol}"));
        }

        slf
    }

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

    pub fn get_symbol_by_name(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get_symbol(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn get_symbol_by_id(&self, id: SymbolId) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get_symbol_by_id(id) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn is_current_scope_global(&self) -> bool {
        matches!(self.scopes.last(), Some(scope) if scope.kind == ScopeKind::Global)
    }

    pub fn is_scope_rooted_in_subroutine(&self) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.kind == ScopeKind::Function)
    }

    pub(crate) fn get_input(&self) -> Option<Vec<Symbol>> {
        let io_input = self.get_io_input();
        if io_input.is_empty() {
            None
        } else {
            Some(io_input)
        }
    }

    pub(crate) fn get_output(&self) -> Option<Vec<Symbol>> {
        let io_ouput = self.get_io_output();
        if io_ouput.is_some() {
            io_ouput
        } else {
            self.get_inferred_output()
        }
    }

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
                    if is_inferred_output_type(&symbol.ty) {
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

fn is_inferred_output_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Bit(_)
            | Type::Int(_, _)
            | Type::UInt(_, _)
            | Type::Float(_, _)
            | Type::Angle(_, _)
            | Type::Complex(_, _)
            | Type::Bool(_)
            | Type::BitArray(_, _)
            | Type::IntArray(_)
            | Type::UIntArray(_)
            | Type::FloatArray(_)
            | Type::AngleArray(_)
            | Type::ComplexArray(_)
            | Type::BoolArray(_)
            | Type::Range
            | Type::Set
    )
}
