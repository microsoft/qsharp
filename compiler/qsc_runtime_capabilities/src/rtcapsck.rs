use crate::{
    single_pass_analysis::{
        AppIdx, ElmntComputeProps, ItemComputeProps, PackageComputeProps, SinglePassAnalyzer,
    },
    RuntimeCapability,
};
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_fir::fir::{
    BlockId, CallableDecl, ItemKind, LocalItemId, Package, PackageId, PackageStore, SpecBody,
    StmtId,
};
use rustc_hash::FxHashSet;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("target does not support callable capabilities: {0}")]
    #[diagnostic(code("Qsc.RuntimeCapabilities.TargetDoesNotSuppoprtCallableCaps"))]
    TargetDoesNotSuppoprtCallableCaps(String, #[label] Span),
    #[error("target does not support statement capabilities: {0}")]
    #[diagnostic(code("Qsc.RuntimeCapabilities.TargetDoesNotSuppoprtStatementCaps"))]
    TargetDoesNotSuppoprtStmtCaps(String, #[label] Span),
}

pub fn check_target_capabilities_compatibility(
    package_store: &PackageStore,
    main_package_id: PackageId,
    target_capabilities: &FxHashSet<RuntimeCapability>,
) -> Vec<Error> {
    let store_compute_props = SinglePassAnalyzer::run(package_store);
    //store_compute_props.persist();
    let main_package_compute_props = store_compute_props
        .get_package(main_package_id)
        .expect("Main package compute properties should exist");
    let main_package = package_store
        .0
        .get(main_package_id)
        .expect("Package should exist");
    let mut errors = Vec::new();
    for (item_id, _) in main_package.items.iter() {
        let mut item_errors = check_item_capabilities_compatibility(
            item_id,
            main_package,
            main_package_compute_props,
            target_capabilities,
        );
        errors.append(&mut item_errors);
    }
    errors
}

fn check_item_capabilities_compatibility(
    item_id: LocalItemId,
    package: &Package,
    package_compute_props: &PackageComputeProps,
    target_capabilities: &FxHashSet<RuntimeCapability>,
) -> Vec<Error> {
    let mut errors = Vec::new();
    let item = package.items.get(item_id).expect("Item should exist");
    if let ItemKind::Callable(callable) = &item.kind {
        let callable_compute_props = match package_compute_props
            .items
            .get(item_id)
            .expect("Item compute properties should exist")
        {
            ItemComputeProps::Callable(callable_compute_props) => callable_compute_props,
            _ => panic!("Item compute properties should be callable compute properties."),
        };

        let all_static_caps = callable_compute_props
            .apps
            .get(AppIdx::all_static_params_idx())
            .expect("Callable application should exist");
        let mut callable_unsupported_caps = Vec::new();
        for rt_cap in all_static_caps.rt_caps.difference(target_capabilities) {
            callable_unsupported_caps.push(rt_cap.clone());
        }
        if !callable_unsupported_caps.is_empty() {
            let callable_unsupported_caps_str = caps_to_str(&callable_unsupported_caps);
            let error = Error::TargetDoesNotSuppoprtCallableCaps(
                callable_unsupported_caps_str,
                callable.name.span,
            );
            errors.push(error);
        }

        // Check each statement.
        let body_block_id = get_callable_implementation_block_id(callable);
        let body_block = package
            .blocks
            .get(body_block_id)
            .expect("Body block should exist");
        for stmt_id in &body_block.stmts {
            let mut stmt_errors = check_statement_capabilities_compatibility(
                *stmt_id,
                package,
                package_compute_props,
                target_capabilities,
            );
            errors.append(&mut stmt_errors);
        }
    }
    errors
}

fn check_statement_capabilities_compatibility(
    stmt_id: StmtId,
    package: &Package,
    package_compute_props: &PackageComputeProps,
    target_capabilities: &FxHashSet<RuntimeCapability>,
) -> Vec<Error> {
    let mut errors = Vec::new();
    let stmt = package.stmts.get(stmt_id).expect("Statement should exist");
    let stmt_compute_props = package_compute_props
        .stmts
        .get(stmt_id)
        .expect("Statement compute properties should exist");

    if let ElmntComputeProps::AppIndependent(independent_compute_props) = stmt_compute_props {
        let mut unsupported_stmt_rt_caps = Vec::new();
        for unsupported_cap in independent_compute_props
            .rt_caps
            .difference(target_capabilities)
        {
            unsupported_stmt_rt_caps.push(unsupported_cap.clone());
        }
        if !unsupported_stmt_rt_caps.is_empty() {
            let stmt_unsupported_caps_str = caps_to_str(&unsupported_stmt_rt_caps);
            let error = Error::TargetDoesNotSuppoprtStmtCaps(stmt_unsupported_caps_str, stmt.span);
            errors.push(error);
        }
    }
    errors
}

fn caps_to_str(rt_caps: &Vec<RuntimeCapability>) -> String {
    let mut caps_str = "{".to_owned();
    for cap in rt_caps {
        caps_str.push_str(&format!("{cap:?}, ").to_owned());
    }
    caps_str.push('}');
    caps_str
}

fn get_callable_implementation_block_id(callable: &CallableDecl) -> BlockId {
    match callable.body.body {
        SpecBody::Impl(_, block_id) => block_id,
        _ => panic!("Is not implementation"),
    }
}
