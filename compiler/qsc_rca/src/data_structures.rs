// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{CallableDecl, ExprId, LocalItemId, NodeId, Pat, PatId, PatKind},
    ty::Ty,
};
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, ops};

/// A callable input element.
#[derive(Debug)]
pub struct CallableInputElement {
    pub pat: PatId,
    pub ty: Ty,
    pub kind: CallableInputElementKind,
}

/// A kind of callable input element.
#[derive(Debug)]
pub enum CallableInputElementKind {
    Discard,
    Node(NodeId),
    Tuple(Vec<PatId>),
}

/// Creates a vector of flattened callable input elements.
pub fn derive_callable_input_elements(
    callable: &CallableDecl,
    pats: &IndexMap<PatId, Pat>,
) -> Vec<CallableInputElement> {
    fn create_input_elements(
        pat_id: PatId,
        pats: &IndexMap<PatId, Pat>,
    ) -> Vec<CallableInputElement> {
        let pat = pats.get(pat_id).expect("pattern should exist");
        match &pat.kind {
            PatKind::Bind(ident) => {
                vec![CallableInputElement {
                    pat: pat_id,
                    ty: pat.ty.clone(),
                    kind: CallableInputElementKind::Node(ident.id),
                }]
            }
            PatKind::Tuple(tuple_pats) => {
                let mut tuple_params = vec![CallableInputElement {
                    pat: pat_id,
                    ty: pat.ty.clone(),
                    kind: CallableInputElementKind::Tuple(tuple_pats.clone()),
                }];
                for tuple_item_pat_id in tuple_pats {
                    let mut tuple_item_params = create_input_elements(*tuple_item_pat_id, pats);
                    tuple_params.append(&mut tuple_item_params);
                }
                tuple_params
            }
            PatKind::Discard => vec![CallableInputElement {
                pat: pat_id,
                ty: pat.ty.clone(),
                kind: CallableInputElementKind::Discard,
            }],
        }
    }

    create_input_elements(callable.input, pats)
}

/// The index corresponding to an input parameter node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputParamIndex(usize);

impl ops::Add<usize> for InputParamIndex {
    type Output = InputParamIndex;

    fn add(self, rhs: usize) -> InputParamIndex {
        InputParamIndex(self.0 + rhs)
    }
}

impl From<usize> for InputParamIndex {
    fn from(value: usize) -> Self {
        InputParamIndex(value)
    }
}

/// An input parameter node.
#[derive(Debug)]
pub struct InputParam {
    pub index: InputParamIndex,
    pub pat: PatId,
    pub ty: Ty,
    pub node: Option<NodeId>,
}

/// Creates a vector of callable input parameters.
pub fn derive_callable_input_params<'a>(
    input_elements: impl Iterator<Item = &'a CallableInputElement>,
) -> Vec<InputParam> {
    let mut input_params = Vec::new();
    let mut param_index = InputParamIndex(0);
    for element in input_elements {
        let maybe_input_param = match &element.kind {
            CallableInputElementKind::Discard => Some(InputParam {
                index: param_index,
                pat: element.pat,
                ty: element.ty.clone(),
                node: None,
            }),
            CallableInputElementKind::Node(node_id) => Some(InputParam {
                index: param_index,
                pat: element.pat,
                ty: element.ty.clone(),
                node: Some(*node_id),
            }),
            CallableInputElementKind::Tuple(_) => None,
        };

        if let Some(input_param) = maybe_input_param {
            input_params.push(input_param);
            param_index.0 += 1;
        }
    }

    input_params
}

/// A represenation of a variable within a callable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableVariable {
    pub node: NodeId,
    pub pat: PatId,
    pub ty: Ty,
    pub kind: CallableVariableKind,
}

impl Ord for CallableVariable {
    fn cmp(&self, other: &Self) -> Ordering {
        match &self.kind {
            CallableVariableKind::InputParam(self_index) => match &other.kind {
                CallableVariableKind::InputParam(other_index) => self_index.0.cmp(&other_index.0),
                CallableVariableKind::Local(_) => Ordering::Less,
            },
            CallableVariableKind::Local(_) => match &other.kind {
                CallableVariableKind::InputParam(_) => Ordering::Greater,
                CallableVariableKind::Local(_) => self.node.cmp(&other.node),
            },
        }
    }
}

impl PartialOrd for CallableVariable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Kinds of callable variables.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum CallableVariableKind {
    Local(ExprId),
    InputParam(InputParamIndex),
}

/// Creates a map of a input parameters.
pub fn derive_callable_input_map<'a>(
    input_params: impl Iterator<Item = &'a InputParam>,
) -> FxHashMap<NodeId, CallableVariable> {
    let mut variable_map = FxHashMap::<NodeId, CallableVariable>::default();
    for param in input_params {
        if let Some(node) = param.node {
            variable_map.insert(
                node,
                CallableVariable {
                    node,
                    pat: param.pat,
                    ty: param.ty.clone(),
                    kind: CallableVariableKind::InputParam(param.index),
                },
            );
        }
    }
    variable_map
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CallableSpecializationId {
    pub callable: LocalItemId,
    pub functor_application: FunctorApplication,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct FunctorApplication {
    pub adjoint: bool,
    // N.B. For the purposes of RCA, we only care about the controlled functor being applied, but not the number of
    // times it was applied.
    pub controlled: bool,
}
