// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::mem::take;

use qsc_hir::{
    assigner::Assigner,
    hir::{Block, Expr, ExprKind, Mutability, Stmt, StmtKind},
    mut_visit::{walk_expr, MutVisitor},
};

use crate::common::{gen_ident, IdentTemplate};

pub(crate) struct ConvertToWSlash<'a> {
    pub(crate) assigner: &'a mut Assigner,
}

// impl ConvertToWSlash<'_> {
//     /// Wraps the given expression in a block expression and returns a mutable reference to the block's statements vector.
//     fn wrap_in_block<'a>(&mut self, expr: &'a mut Expr) -> (&'a mut Expr, &'a mut Vec<Stmt>) {
//         let base_expr = take(expr);

//         let block_expr = Expr {
//             id: self.assigner.next_node(),
//             span: base_expr.span,
//             ty: base_expr.ty.clone(),
//             kind: ExprKind::Block(Block {
//                 id: self.assigner.next_node(),
//                 span: base_expr.span,
//                 ty: base_expr.ty.clone(),
//                 stmts: vec![Stmt {
//                     id: self.assigner.next_node(),
//                     span: base_expr.span,
//                     kind: StmtKind::Expr(base_expr),
//                 }],
//             }),
//         };
//         *expr = block_expr;

//         // Extract a mutable reference to the statements vector
//         if let ExprKind::Block(block) = &mut expr.kind {
//             return (&mut base_expr, &mut block.stmts);
//         }

//         unreachable!("Expected block expression to contain a block");
//     }
// }

impl MutVisitor for ConvertToWSlash<'_> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);
        if let ExprKind::Assign(lhs, rhs) = &mut expr.kind {
            let mut indices = vec![];
            let mut current = lhs;
            let mut curr_kind = &mut current.kind;

            // Traverse the LHS and collect mutable references to index expressions
            while let ExprKind::Index(array, index) = curr_kind {
                indices.push(index);
                current = array;
                curr_kind = &mut current.kind;
            }

            // If the LHS is not an array index, return early
            if indices.is_empty() {
                return;
            }

            // Verify that the ultimate LHS is a Var
            if matches!(curr_kind, ExprKind::Var(_, _)) {
                let base_array = Box::new(Expr {
                    id: current.id,
                    span: current.span,
                    ty: current.ty.clone(),
                    kind: curr_kind.clone(),
                });

                let index_ids: Vec<(IdentTemplate, Box<Expr>)> = indices
                    .into_iter()
                    .rev()
                    .map(|index| {
                        (
                            gen_ident(self.assigner, "index", index.ty.clone(), index.span),
                            take(index),
                        )
                    })
                    .collect();

                // Create local binding statements for each index
                let mut stmts = vec![];
                let mut indexes = vec![];
                for (id, index) in index_ids {
                    stmts.push(id.gen_id_init(Mutability::Immutable, *index, self.assigner));
                    indexes.push(id);
                }

                // Construct the nested `w/` expressions
                let mut nested_expr = rhs.clone();
                for (i, id) in indexes.iter().enumerate().rev() {
                    let target_expr = if i == 0 {
                        // For the outermost `w/=`, use the variable name
                        let mut array_expr = base_array.clone();
                        array_expr.id = self.assigner.next_node();
                        array_expr
                    } else {
                        // For inner `w/`, use the array indexing
                        let mut array_expr = base_array.clone();
                        array_expr.id = self.assigner.next_node();
                        for target_index in &indexes[..i] {
                            array_expr = Box::new(Expr {
                                id: self.assigner.next_node(),
                                span: expr.span,     // ToDo: not correct
                                ty: expr.ty.clone(), // ToDo: not correct
                                kind: ExprKind::Index(
                                    array_expr,
                                    Box::new(target_index.gen_local_ref(self.assigner)),
                                ),
                            });
                        }
                        array_expr
                    };

                    nested_expr = Box::new(Expr {
                        id: self.assigner.next_node(),
                        span: expr.span,     // ToDo: not correct
                        ty: expr.ty.clone(), // ToDo: not correct
                        kind: if i == 0 {
                            // Outermost `w/=`
                            ExprKind::AssignIndex(
                                target_expr,
                                Box::new(id.gen_local_ref(self.assigner)),
                                nested_expr,
                            )
                        } else {
                            // Inner `w/`
                            ExprKind::UpdateIndex(
                                target_expr,
                                Box::new(id.gen_local_ref(self.assigner)),
                                nested_expr,
                            )
                        },
                    });
                }

                // Add the final `w/=` expression as a statement
                stmts.push(Stmt {
                    id: self.assigner.next_node(),
                    span: expr.span,
                    kind: StmtKind::Expr(*nested_expr),
                });

                // Wrap all statements in a block expression
                let block_expr = Expr {
                    id: self.assigner.next_node(),
                    span: expr.span,
                    ty: expr.ty.clone(),
                    kind: ExprKind::Block(Block {
                        id: self.assigner.next_node(),
                        span: expr.span,
                        ty: expr.ty.clone(),
                        stmts,
                    }),
                };

                // Replace the original expression with the block expression
                *expr = block_expr;
            }
        }
    }
}
