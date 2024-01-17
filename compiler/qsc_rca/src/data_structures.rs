// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{CallableDecl, NodeId, Pat, PatId, PatKind},
    ty::Ty,
};
use std::ops::Deref;

/// An element of an input parameter pattern.
pub enum InputParamElmnt {
    Discard(PatId, Ty),
    Node(PatId, NodeId, Ty),
    Tuple(PatId, Vec<PatId>, Ty),
}

pub struct FlattenedInputParamsElmnts(Vec<InputParamElmnt>);

impl Deref for FlattenedInputParamsElmnts {
    type Target = Vec<InputParamElmnt>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FlattenedInputParamsElmnts {
    /// Creates a flattened vector of input parameters elements.
    pub fn from_callable(callable: &CallableDecl, pats: &IndexMap<PatId, Pat>) -> Self {
        fn as_vector(pat_id: PatId, pats: &IndexMap<PatId, Pat>) -> Vec<InputParamElmnt> {
            let pat = pats.get(pat_id).expect("pattern should exist");
            match &pat.kind {
                PatKind::Bind(ident) => {
                    vec![InputParamElmnt::Node(pat_id, ident.id, pat.ty.clone())]
                }
                PatKind::Tuple(tuple_pats) => {
                    let mut tuple_params = vec![InputParamElmnt::Tuple(
                        pat_id,
                        tuple_pats.clone(),
                        pat.ty.clone(),
                    )];
                    for tuple_item_pat_id in tuple_pats {
                        let mut tuple_item_params = as_vector(*tuple_item_pat_id, pats);
                        tuple_params.append(&mut tuple_item_params);
                    }
                    tuple_params
                }
                PatKind::Discard => vec![InputParamElmnt::Discard(pat_id, pat.ty.clone())],
            }
        }

        let flat_pat = as_vector(callable.input, pats);
        Self(flat_pat)
    }
}
