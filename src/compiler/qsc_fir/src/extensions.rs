// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    fir::{CallableDecl, LocalVarId, Package, Pat, PatId, PatKind},
    ty::Ty,
};
use qsc_data_structures::index_map::IndexMap;

/// The index corresponding to an input parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputParamIndex(usize);

impl From<InputParamIndex> for usize {
    fn from(value: InputParamIndex) -> Self {
        value.0
    }
}

impl From<usize> for InputParamIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// An input parameter.
#[derive(Clone, Debug)]
pub struct InputParam {
    pub index: InputParamIndex,
    pub pat: PatId,
    pub ty: Ty,
    pub var: Option<LocalVarId>,
}

impl Package {
    #[must_use]
    pub fn derive_callable_input_params(&self, callable: &CallableDecl) -> Vec<InputParam> {
        let input_elements = derive_callable_input_pattern_elements(callable, &self.pats);
        let mut input_params = Vec::new();
        let mut param_index = InputParamIndex(0);
        for element in input_elements {
            let maybe_input_param = match &element.kind {
                InputPatternElementKind::Discard => Some(InputParam {
                    index: param_index,
                    pat: element.pat,
                    ty: element.ty.clone(),
                    var: None,
                }),
                InputPatternElementKind::Ident(local_var_id) => Some(InputParam {
                    index: param_index,
                    pat: element.pat,
                    ty: element.ty.clone(),
                    var: Some(*local_var_id),
                }),
                InputPatternElementKind::Tuple => None,
            };

            if let Some(input_param) = maybe_input_param {
                input_params.push(input_param);
                param_index.0 += 1;
            }
        }

        input_params
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
    Ident(LocalVarId),
    Tuple,
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
                    kind: InputPatternElementKind::Ident(ident.id),
                }]
            }
            PatKind::Tuple(tuple_pats) => {
                let mut tuple_params = vec![InputPatternElement {
                    pat: pat_id,
                    ty: pat.ty.clone(),
                    kind: InputPatternElementKind::Tuple,
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
