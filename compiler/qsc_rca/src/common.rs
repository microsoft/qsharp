// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indenter::Indented;
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        CallableDecl, ExprId, ExprKind, Functor, ItemId, NodeId, PackageId, PackageStoreLookup,
        Pat, PatId, PatKind, Res, SpecDecl, StoreExprId, StoreItemId, UnOp,
    },
    ty::Ty,
};
use rustc_hash::FxHashMap;
use std::fmt::{Debug, Formatter};

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

pub fn derive_specialization_input_params(
    spec_decl: &SpecDecl,
    callable_input_params: &Vec<InputParam>,
    pats: &IndexMap<PatId, Pat>,
) -> Vec<InputParam> {
    let spec_input_element = derive_specialization_input_pattern_element(spec_decl, pats);
    if let Some(spec_input_element) = spec_input_element {
        // Insert the specialization input parameter at the beginning of the new input parameters vector.
        let spec_input_param = InputParam {
            index: InputParamIndex(0),
            pat: spec_input_element.pat,
            ty: spec_input_element.ty.clone(),
            node: match &spec_input_element.kind {
                InputPatternElementKind::Discard => None,
                InputPatternElementKind::Node(node_id) => Some(*node_id),
                InputPatternElementKind::Tuple(_) => {
                    panic!("specialization inputs are not expected to be tuples")
                }
            },
        };
        let mut spec_input_params = vec![spec_input_param];

        // Now, insert the callable input parameters shifting the input parameter index.
        for input_param in callable_input_params {
            spec_input_params.push(InputParam {
                index: InputParamIndex(input_param.index.0 + 1),
                pat: input_param.pat,
                ty: input_param.ty.clone(),
                node: input_param.node,
            })
        }
        spec_input_params
    } else {
        callable_input_params.clone()
    }
}

/// A represenation of a local symbol.
#[derive(Debug)]
pub struct Local {
    pub node: NodeId,
    pub pat: PatId,
    pub ty: Ty,
    pub kind: LocalKind,
}

/// Kinds of local symbols.
#[derive(Debug)]
pub enum LocalKind {
    /// An input parameter with its associated index in the context of a particular specialization.
    InputParam(InputParamIndex),
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
pub struct GlobalSpecializationId {
    pub callable: StoreItemId,
    pub specialization: SpecializationKind,
}

impl From<(StoreItemId, SpecializationKind)> for GlobalSpecializationId {
    fn from(value: (StoreItemId, SpecializationKind)) -> Self {
        Self {
            callable: value.0,
            specialization: value.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpecializationKind {
    Body,
    Adj,
    Ctl,
    CtlAdj,
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

/// Creates an input pattern element if the specialization input is `Some`.
fn derive_specialization_input_pattern_element(
    spec_decl: &SpecDecl,
    pats: &IndexMap<PatId, Pat>,
) -> Option<InputPatternElement> {
    spec_decl.input.map(|pat_id| {
        let pat = pats.get(pat_id).expect("pattern should exist");
        match &pat.kind {
            PatKind::Bind(ident) => InputPatternElement {
                pat: pat_id,
                ty: pat.ty.clone(),
                kind: InputPatternElementKind::Node(ident.id),
            },
            PatKind::Discard => InputPatternElement {
                pat: pat_id,
                ty: pat.ty.clone(),
                kind: InputPatternElementKind::Discard,
            },
            PatKind::Tuple(_) => panic!("specialization inputs are not expected to be tuples"),
        }
    })
}

#[derive(Debug)]
pub struct Callee {
    callable: StoreItemId,
    spec_functor: SpecFunctor,
}

#[derive(Debug, Default)]
pub struct SpecFunctor {
    adjoint: bool,
    controlled: u8,
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
        callable: (package_id, item_id.item).into(),
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
            callable: callee.callable,
            spec_functor,
        }
    })
}
