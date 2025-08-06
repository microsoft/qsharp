// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod dyn_array_ref;
mod static_array_ref;

use super::check;
use expect_test::expect;

#[test]
fn too_many_indices_in_indexed_assignment() {
    check(
        r#"
        array[float[32], 3, 2] multiDim = {{1.1, 1.2}, {2.1, 2.2}, {3.1, 3.2}};
        multiDim[1, 1, 3] = 2.3;
        "#,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-80]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-80]:
                            symbol_id: 8
                            ty_span: [9-31]
                            init_expr: Expr [43-79]:
                                ty: array[float[32], 3, 2]
                                kind: Lit:     array:
                                        Expr [44-54]:
                                            ty: array[float[32], 2]
                                            kind: Lit:     array:
                                                    Expr [45-48]:
                                                        ty: float[32]
                                                        kind: Cast [45-48]:
                                                            ty: float[32]
                                                            expr: Expr [45-48]:
                                                                ty: const float
                                                                kind: Lit: Float(1.1)
                                                            kind: Implicit
                                                    Expr [50-53]:
                                                        ty: float[32]
                                                        kind: Cast [50-53]:
                                                            ty: float[32]
                                                            expr: Expr [50-53]:
                                                                ty: const float
                                                                kind: Lit: Float(1.2)
                                                            kind: Implicit
                                        Expr [56-66]:
                                            ty: array[float[32], 2]
                                            kind: Lit:     array:
                                                    Expr [57-60]:
                                                        ty: float[32]
                                                        kind: Cast [57-60]:
                                                            ty: float[32]
                                                            expr: Expr [57-60]:
                                                                ty: const float
                                                                kind: Lit: Float(2.1)
                                                            kind: Implicit
                                                    Expr [62-65]:
                                                        ty: float[32]
                                                        kind: Cast [62-65]:
                                                            ty: float[32]
                                                            expr: Expr [62-65]:
                                                                ty: const float
                                                                kind: Lit: Float(2.2)
                                                            kind: Implicit
                                        Expr [68-78]:
                                            ty: array[float[32], 2]
                                            kind: Lit:     array:
                                                    Expr [69-72]:
                                                        ty: float[32]
                                                        kind: Cast [69-72]:
                                                            ty: float[32]
                                                            expr: Expr [69-72]:
                                                                ty: const float
                                                                kind: Lit: Float(3.1)
                                                            kind: Implicit
                                                    Expr [74-77]:
                                                        ty: float[32]
                                                        kind: Cast [74-77]:
                                                            ty: float[32]
                                                            expr: Expr [74-77]:
                                                                ty: const float
                                                                kind: Lit: Float(3.2)
                                                            kind: Implicit
                    Stmt [89-113]:
                        annotations: <empty>
                        kind: AssignStmt [89-113]:
                            lhs: Expr [89-102]:
                                ty: unknown
                                kind: Err
                            rhs: Expr [89-113]:
                                ty: unknown
                                kind: Err

            [Qasm.Lowerer.CannotIndexType

              x cannot index variables of type float[32]
               ,-[test:3:24]
             2 |         array[float[32], 3, 2] multiDim = {{1.1, 1.2}, {2.1, 2.2}, {3.1, 3.2}};
             3 |         multiDim[1, 1, 3] = 2.3;
               :                        ^
             4 |         
               `----
            ]"#]],
    );
}
