// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indenter::Indented;
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{CallableDecl, ExprId, NodeId, Pat, PatId, PatKind, SpecDecl, StoreItemId},
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
    /// A local symbol with the lastest expression associated to it.
    Local(ExprId),
}

pub type LocalsMap = FxHashMap<NodeId, Local>;

pub fn initalize_locals_map(input_params: &Vec<InputParam>) -> LocalsMap {
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
