// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{CallableDecl, NodeId, Pat, PatId, PatKind},
    ty::Ty,
};
use rustc_hash::FxHashMap;
use std::ops::Deref;

/// An element of an input parameter pattern.
#[derive(Debug)]
pub struct InputParamElmnt {
    pub pat: PatId,
    pub ty: Ty,
    pub kind: InputParamElmntKind,
}

/// A kind of element in an input parameter pattern.
#[derive(Debug)]
pub enum InputParamElmntKind {
    Discard,
    Node(NodeId),
    Tuple(Vec<PatId>),
}

/// A flat list of input parameter elements.
#[derive(Debug)]
pub struct FlattenedInputParamsElmnts(Vec<InputParamElmnt>);

impl Deref for FlattenedInputParamsElmnts {
    type Target = Vec<InputParamElmnt>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FlattenedInputParamsElmnts {
    /// Creates a flattened vector of input parameters elements.
    pub fn new(callable: &CallableDecl, pats: &IndexMap<PatId, Pat>) -> Self {
        fn as_vector(pat_id: PatId, pats: &IndexMap<PatId, Pat>) -> Vec<InputParamElmnt> {
            let pat = pats.get(pat_id).expect("pattern should exist");
            match &pat.kind {
                PatKind::Bind(ident) => {
                    vec![InputParamElmnt {
                        pat: pat_id,
                        ty: pat.ty.clone(),
                        kind: InputParamElmntKind::Node(ident.id),
                    }]
                }
                PatKind::Tuple(tuple_pats) => {
                    let mut tuple_params = vec![InputParamElmnt {
                        pat: pat_id,
                        ty: pat.ty.clone(),
                        kind: InputParamElmntKind::Tuple(tuple_pats.clone()),
                    }];
                    for tuple_item_pat_id in tuple_pats {
                        let mut tuple_item_params = as_vector(*tuple_item_pat_id, pats);
                        tuple_params.append(&mut tuple_item_params);
                    }
                    tuple_params
                }
                PatKind::Discard => vec![InputParamElmnt {
                    pat: pat_id,
                    ty: pat.ty.clone(),
                    kind: InputParamElmntKind::Discard,
                }],
            }
        }

        let flat_pat = as_vector(callable.input, pats);
        Self(flat_pat)
    }
}

/// The index corresponding to an input parameter node.
#[derive(Clone, Copy, Debug)]
pub struct InputParamIdx(usize);

/// An input parameter node.
#[derive(Debug)]
pub struct InputParam {
    pub idx: InputParamIdx,
    pub pat: PatId,
    pub ty: Ty,
    pub node: Option<NodeId>,
}

/// A vector of input parameters.
#[derive(Debug)]
pub struct InputParams(Vec<InputParam>);

impl Deref for InputParams {
    type Target = Vec<InputParam>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl InputParams {
    pub fn new(input_params_elmnts: &FlattenedInputParamsElmnts) -> Self {
        let mut input_params = Vec::new();
        let mut param_idx = InputParamIdx(0);
        for elmnt in input_params_elmnts.iter() {
            let maybe_input_param = match &elmnt.kind {
                InputParamElmntKind::Discard => Some(InputParam {
                    idx: param_idx,
                    pat: elmnt.pat,
                    ty: elmnt.ty.clone(),
                    node: None,
                }),
                InputParamElmntKind::Node(node_id) => Some(InputParam {
                    idx: param_idx,
                    pat: elmnt.pat,
                    ty: elmnt.ty.clone(),
                    node: Some(*node_id),
                }),
                InputParamElmntKind::Tuple(_) => None,
            };

            if let Some(input_param) = maybe_input_param {
                input_params.push(input_param);
                param_idx.0 += 1;
            }
        }

        Self(input_params)
    }
}

/// A represenation of a variable within a callable.
#[derive(Debug)]
pub struct CallableVar {
    pub node: NodeId,
    pub pat: PatId,
    pub ty: Ty,
    pub kind: CallableVarKind,
}

/// Kinds of callable variables.
#[derive(Debug)]
pub enum CallableVarKind {
    Local,
    InputParam(InputParamIdx),
}

/// A map of variables associated to a callable.
#[derive(Debug)]
pub struct CallableVarMap(FxHashMap<NodeId, CallableVar>);

impl CallableVarMap {
    pub fn new(input_params: &InputParams) -> Self {
        let mut var_map = FxHashMap::<NodeId, CallableVar>::default();
        for param in input_params.iter() {
            if let Some(node) = param.node {
                var_map.insert(
                    node,
                    CallableVar {
                        node,
                        pat: param.pat,
                        ty: param.ty.clone(),
                        kind: CallableVarKind::InputParam(param.idx),
                    },
                );
            }
        }
        Self(var_map)
    }
}
