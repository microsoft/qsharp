// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::QuantumProperties;
use indenter::Indented;
use qsc_data_structures::{functors::FunctorApp, index_map::IndexMap};
use qsc_fir::{
    fir::{
        CallableDecl, ExprId, ExprKind, Functor, ItemId, LocalVarId, PackageId, PackageLookup, Pat,
        PatId, PatKind, Res, StoreItemId, UnOp,
    },
    ty::{FunctorSetValue, Ty},
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
    pub var: Option<LocalVarId>,
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
                var: None,
            }),
            InputPatternElementKind::Ident(local_var_id) => Some(InputParam {
                index: param_index,
                pat: element.pat,
                ty: element.ty.clone(),
                var: Some(*local_var_id),
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
    pub var: LocalVarId,
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
    fn find(&self, local_var_id: LocalVarId) -> Option<&Local>;

    fn get(&self, local_var_id: LocalVarId) -> &Local {
        self.find(local_var_id).expect("local should exist")
    }
}

impl LocalsLookup for FxHashMap<LocalVarId, Local> {
    fn find(&self, local_var_id: LocalVarId) -> Option<&Local> {
        self.get(&local_var_id)
    }
}

pub fn initalize_locals_map(input_params: &Vec<InputParam>) -> FxHashMap<LocalVarId, Local> {
    let mut locals_map = FxHashMap::<LocalVarId, Local>::default();
    for param in input_params {
        if let Some(id) = param.var {
            locals_map.insert(
                id,
                Local {
                    var: id,
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
    pub functor_set_value: FunctorSetValue,
}

impl From<(StoreItemId, FunctorSetValue)> for GlobalSpecId {
    fn from(value: (StoreItemId, FunctorSetValue)) -> Self {
        Self {
            callable: value.0,
            functor_set_value: value.1,
        }
    }
}

pub trait FunctorAppExt {
    fn functor_set_value(&self) -> FunctorSetValue;
}

impl FunctorAppExt for FunctorApp {
    fn functor_set_value(&self) -> FunctorSetValue {
        let adjoint = self.adjoint;
        let controlled = self.controlled > 0;
        match (adjoint, controlled) {
            (false, false) => FunctorSetValue::Empty,
            (true, false) => FunctorSetValue::Adj,
            (false, true) => FunctorSetValue::Ctl,
            (true, true) => FunctorSetValue::CtlAdj,
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
    Ident(LocalVarId),
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
                    kind: InputPatternElementKind::Ident(ident.id),
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
    pub functor_app: FunctorApp,
}

/// Tries to uniquely resolve the callable specialization referenced in a callee expression.
pub fn try_resolve_callee(
    expr_id: ExprId,
    package_id: PackageId,
    package: &impl PackageLookup,
    locals_map: &impl LocalsLookup,
) -> Option<Callee> {
    // This is a best effort attempt to resolve a callee that currently only supports resolving
    // global callables or locals that eventually resolve to global callables.
    let expr = package.get_expr(expr_id);
    match &expr.kind {
        ExprKind::UnOp(operator, operand_expr_id) => {
            try_resolve_un_op_callee(*operator, *operand_expr_id, package_id, package, locals_map)
        }
        ExprKind::Var(res, _) => match res {
            Res::Item(item_id) => resolve_item_callee(package_id, *item_id),
            Res::Local(local_var_id) => {
                try_resolve_local_callee(*local_var_id, package_id, package, locals_map)
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
        functor_app: FunctorApp::default(),
    })
}

fn try_resolve_local_callee(
    local_var_id: LocalVarId,
    package_id: PackageId,
    package: &impl PackageLookup,
    locals_map: &impl LocalsLookup,
) -> Option<Callee> {
    // This is a best effort attempt to resolve a callee.
    locals_map
        .find(local_var_id)
        .and_then(|local| match local.kind {
            LocalKind::Immutable(expr_id) => {
                try_resolve_callee(expr_id, package_id, package, locals_map)
            }
            _ => None,
        })
}

fn try_resolve_un_op_callee(
    operator: UnOp,
    expr_id: ExprId,
    package_id: PackageId,
    package: &impl PackageLookup,
    locals_map: &impl LocalsLookup,
) -> Option<Callee> {
    // This is a best effort attempt to resolve a callee.
    let UnOp::Functor(functor_operator) = operator else {
        panic!("unary operator is expected to be a functor for a callee expression")
    };

    try_resolve_callee(expr_id, package_id, package, locals_map).map(|callee| {
        let spec_functor = match functor_operator {
            Functor::Adj => FunctorApp {
                adjoint: !callee.functor_app.adjoint,
                controlled: callee.functor_app.controlled,
            },
            Functor::Ctl => FunctorApp {
                adjoint: callee.functor_app.adjoint,
                controlled: callee.functor_app.controlled + 1,
            },
        };
        Callee {
            item: callee.item,
            functor_app: spec_functor,
        }
    })
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
