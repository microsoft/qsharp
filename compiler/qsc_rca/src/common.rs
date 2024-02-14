// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::QuantumProperties;
use indenter::Indented;
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        CallableDecl, ExprId, ExprKind, Functor, Global, ItemId, NodeId, PackageId, PackageStore,
        PackageStoreLookup, Pat, PatId, PatKind, Res, StoreExprId, StoreItemId, UnOp,
    },
    ty::Ty,
};
use rustc_hash::FxHashMap;
use std::fmt::{Debug, Formatter};

use crate::{ComputeKind, ValueKind};

/// Aggregates two compute kind structures returning the resulting compute kind.
#[must_use]
pub fn aggregate_compute_kind(basis: ComputeKind, delta: &ComputeKind) -> ComputeKind {
    let ComputeKind::Quantum(delta_quantum_properties) = delta else {
        // A classical compute kind has nothing to aggregate so just return the base with no changes.
        return basis;
    };

    // Determine the aggregated runtime features.
    let runtime_features = match basis {
        ComputeKind::Classical => delta_quantum_properties.runtime_features,
        ComputeKind::Quantum(ref basis_quantum_properties) => {
            basis_quantum_properties.runtime_features | delta_quantum_properties.runtime_features
        }
    };

    // Determine the aggregated value kind.
    let value_kind = match basis {
        ComputeKind::Classical => delta_quantum_properties.value_kind,
        ComputeKind::Quantum(basis_quantum_properties) => aggregate_value_kind(
            basis_quantum_properties.value_kind,
            &delta_quantum_properties.value_kind,
        ),
    };

    // Return the aggregated compute kind.
    ComputeKind::Quantum(QuantumProperties {
        runtime_features,
        value_kind,
    })
}

#[must_use]
pub fn aggregate_value_kind(basis: ValueKind, delta: &ValueKind) -> ValueKind {
    match delta {
        ValueKind::Static => basis,
        ValueKind::Dynamic => ValueKind::Dynamic,
    }
}

/// The index corresponding to an input parameter node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputParamIndex(usize);

/// An input parameter node.
#[derive(Clone, Debug)]
pub struct InputParam {
    pub index: InputParamIndex,
    pub pat: PatId,
    pub ty: Ty,
    pub node: Option<NodeId>,
}

pub fn derive_callable_input_params(
    callable: &CallableDecl,
    pats: &IndexMap<PatId, Pat>,
) -> Vec<InputParam> {
    let input_elements = derive_callable_input_pattern_elements(callable, pats);
    let mut input_params = Vec::new();
    let mut param_index = InputParamIndex(0);
    for element in input_elements {
        let maybe_input_param = match &element.kind {
            InputPatternElementKind::Discard => Some(InputParam {
                index: param_index,
                pat: element.pat,
                ty: element.ty.clone(),
                node: None,
            }),
            InputPatternElementKind::Node(node_id) => Some(InputParam {
                index: param_index,
                pat: element.pat,
                ty: element.ty.clone(),
                node: Some(*node_id),
            }),
            InputPatternElementKind::Tuple(_) => None,
        };

        if let Some(input_param) = maybe_input_param {
            input_params.push(input_param);
            param_index.0 += 1;
        }
    }

    input_params
}

/// A represenation of a local symbol.
#[derive(Clone, Debug)]
pub struct Local {
    pub node: NodeId,
    pub pat: PatId,
    pub ty: Ty,
    pub kind: LocalKind,
}

/// Kinds of local symbols.
#[derive(Clone, Debug)]
pub enum LocalKind {
    /// An input parameter with its associated index.
    InputParam(InputParamIndex),
    /// A specialization input (i.e. control qubits).
    SpecInput,
    /// An immutable binding with the expression associated to it.
    Immutable(ExprId),
    /// A mutable binding.
    Mutable,
}

pub trait LocalsLookup {
    fn find(&self, node_id: NodeId) -> Option<&Local>;

    fn get(&self, node_id: NodeId) -> &Local {
        self.find(node_id).expect("local should exist")
    }
}

impl LocalsLookup for FxHashMap<NodeId, Local> {
    fn find(&self, node_id: NodeId) -> Option<&Local> {
        self.get(&node_id)
    }
}

pub fn initalize_locals_map(input_params: &Vec<InputParam>) -> FxHashMap<NodeId, Local> {
    let mut locals_map = FxHashMap::<NodeId, Local>::default();
    for param in input_params {
        if let Some(node) = param.node {
            locals_map.insert(
                node,
                Local {
                    node,
                    pat: param.pat,
                    ty: param.ty.clone(),
                    kind: LocalKind::InputParam(param.index),
                },
            );
        }
    }
    locals_map
}

#[derive(Clone, Copy, Debug)]
pub struct GlobalSpecId {
    pub callable: StoreItemId,
    pub specialization: SpecKind,
}

impl From<(StoreItemId, SpecKind)> for GlobalSpecId {
    fn from(value: (StoreItemId, SpecKind)) -> Self {
        Self {
            callable: value.0,
            specialization: value.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpecKind {
    Body,
    Adj,
    Ctl,
    CtlAdj,
}

impl From<&SpecFunctor> for SpecKind {
    fn from(value: &SpecFunctor) -> Self {
        let adjoint = value.adjoint;
        let controlled = value.controlled > 0;
        match (adjoint, controlled) {
            (false, false) => Self::Body,
            (true, false) => Self::Adj,
            (false, true) => Self::Ctl,
            (true, true) => Self::CtlAdj,
        }
    }
}

/// An element related to an input pattern.
#[derive(Debug)]
struct InputPatternElement {
    pub pat: PatId,
    pub ty: Ty,
    pub kind: InputPatternElementKind,
}

/// Kinds of input pattern elements.
#[derive(Debug)]
enum InputPatternElementKind {
    Discard,
    Node(NodeId),
    Tuple(Vec<PatId>),
}

/// Creates a vector of flattened input pattern elements.
fn derive_callable_input_pattern_elements(
    callable: &CallableDecl,
    pats: &IndexMap<PatId, Pat>,
) -> Vec<InputPatternElement> {
    fn create_input_elements(
        pat_id: PatId,
        pats: &IndexMap<PatId, Pat>,
    ) -> Vec<InputPatternElement> {
        let pat = pats.get(pat_id).expect("pattern should exist");
        match &pat.kind {
            PatKind::Bind(ident) => {
                vec![InputPatternElement {
                    pat: pat_id,
                    ty: pat.ty.clone(),
                    kind: InputPatternElementKind::Node(ident.id),
                }]
            }
            PatKind::Tuple(tuple_pats) => {
                let mut tuple_params = vec![InputPatternElement {
                    pat: pat_id,
                    ty: pat.ty.clone(),
                    kind: InputPatternElementKind::Tuple(tuple_pats.clone()),
                }];
                for tuple_item_pat_id in tuple_pats {
                    let mut tuple_item_params = create_input_elements(*tuple_item_pat_id, pats);
                    tuple_params.append(&mut tuple_item_params);
                }
                tuple_params
            }
            PatKind::Discard => vec![InputPatternElement {
                pat: pat_id,
                ty: pat.ty.clone(),
                kind: InputPatternElementKind::Discard,
            }],
        }
    }

    create_input_elements(callable.input, pats)
}

#[derive(Debug)]
pub struct Callee {
    pub item: StoreItemId,
    pub spec_functor: SpecFunctor,
}

#[derive(Debug, Default)]
pub struct SpecFunctor {
    pub adjoint: bool,
    pub controlled: u8,
}

/// Tries to uniquely resolve the callable specialization referenced in a callee expression.
pub fn try_resolve_callee(
    expr_id: StoreExprId,
    locals_map: &impl LocalsLookup,
    package_store: &impl PackageStoreLookup,
) -> Option<Callee> {
    // This is a best effort attempt to resolve a callee that currently only supports resolving
    // global callables or locals that eventually resolve to global callables.
    let expr = package_store.get_expr(expr_id);
    match &expr.kind {
        ExprKind::UnOp(operator, operand_expr_id) => try_resolve_un_op_callee(
            *operator,
            (expr_id.package, *operand_expr_id).into(),
            locals_map,
            package_store,
        ),
        ExprKind::Var(res, _) => match res {
            Res::Item(item_id) => resolve_item_callee(expr_id.package, *item_id),
            Res::Local(node_id) => {
                try_resolve_local_callee(expr_id.package, *node_id, locals_map, package_store)
            }
            Res::Err => panic!("callee resolution should not be an error"),
        },
        // N.B. More complex callee expressions might require evaluation so we don't try to resolve them at compile
        // time.
        _ => None,
    }
}

fn resolve_item_callee(call_package_id: PackageId, item_id: ItemId) -> Option<Callee> {
    let package_id = item_id.package.unwrap_or(call_package_id);
    Some(Callee {
        item: (package_id, item_id.item).into(),
        spec_functor: SpecFunctor::default(),
    })
}

fn try_resolve_local_callee(
    call_package_id: PackageId,
    node_id: NodeId,
    locals_map: &impl LocalsLookup,
    package_store: &impl PackageStoreLookup,
) -> Option<Callee> {
    // This is a best effort attempt to resolve a callee.
    locals_map.find(node_id).and_then(|local| match local.kind {
        LocalKind::Immutable(expr_id) => {
            try_resolve_callee((call_package_id, expr_id).into(), locals_map, package_store)
        }
        _ => None,
    })
}

fn try_resolve_un_op_callee(
    operator: UnOp,
    expr_id: StoreExprId,
    locals_map: &impl LocalsLookup,
    package_store: &impl PackageStoreLookup,
) -> Option<Callee> {
    // This is a best effort attempt to resolve a callee.
    let UnOp::Functor(functor_operator) = operator else {
        panic!("unary operator is expected to be a functor for a callee expression")
    };

    try_resolve_callee(expr_id, locals_map, package_store).map(|callee| {
        let spec_functor = match functor_operator {
            Functor::Adj => SpecFunctor {
                adjoint: !callee.spec_functor.adjoint,
                controlled: callee.spec_functor.controlled,
            },
            Functor::Ctl => SpecFunctor {
                adjoint: callee.spec_functor.adjoint,
                controlled: callee.spec_functor.controlled + 1,
            },
        };
        Callee {
            item: callee.item,
            spec_functor,
        }
    })
}

pub trait PackageStoreLookupExtension {
    fn get_callable(&self, id: StoreItemId) -> &CallableDecl;
}

impl PackageStoreLookupExtension for PackageStore {
    fn get_callable(&self, id: StoreItemId) -> &CallableDecl {
        let global = self.get_global(id).expect("global should exist");
        let Global::Callable(callble_decl) = global else {
            panic!("global should be callable");
        };

        callble_decl
    }
}

pub fn set_indentation<'a, 'b>(
    indent: Indented<'a, Formatter<'b>>,
    level: usize,
) -> Indented<'a, Formatter<'b>> {
    match level {
        0 => indent.with_str(""),
        1 => indent.with_str("    "),
        2 => indent.with_str("        "),
        _ => unimplemented!("intentation level not supported"),
    }
}
