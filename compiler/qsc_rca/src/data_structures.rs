// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{CallableDecl, NodeId, Pat, PatId, PatKind},
    ty::Ty,
};
use std::ops;

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
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Debug)]
pub struct CallableVariable {
    pub node: NodeId,
    pub pat: PatId,
    pub ty: Ty,
    pub kind: CallableVariableKind,
}

/// Kinds of callable variables.
#[derive(Clone, Debug, Copy)]
pub enum CallableVariableKind {
    #[allow(dead_code)] // TODO (ceasarzc): Remove.
    Local,
    InputParam(InputParamIndex),
}
