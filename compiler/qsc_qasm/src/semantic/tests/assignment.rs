// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use super::check;

#[allow(clippy::too_many_lines)]
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
                statements:
                    Stmt [9-80]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-80]:
                            symbol_id: 8
                            ty_span: [9-31]
                            init_expr: Expr [43-79]:
                                ty: FloatArray(Some(32), Two(3, 2))
                                kind: Lit:     array:
                                        Expr [44-54]:
                                            ty: FloatArray(Some(32), One(2))
                                            kind: Lit:     array:
                                                    Expr [45-48]:
                                                        ty: float[32]
                                                        kind: Cast [0-0]:
                                                            ty: float[32]
                                                            expr: Expr [45-48]:
                                                                ty: const float
                                                                kind: Lit: Float(1.1)
                                                    Expr [50-53]:
                                                        ty: float[32]
                                                        kind: Cast [0-0]:
                                                            ty: float[32]
                                                            expr: Expr [50-53]:
                                                                ty: const float
                                                                kind: Lit: Float(1.2)
                                        Expr [56-66]:
                                            ty: FloatArray(Some(32), One(2))
                                            kind: Lit:     array:
                                                    Expr [57-60]:
                                                        ty: float[32]
                                                        kind: Cast [0-0]:
                                                            ty: float[32]
                                                            expr: Expr [57-60]:
                                                                ty: const float
                                                                kind: Lit: Float(2.1)
                                                    Expr [62-65]:
                                                        ty: float[32]
                                                        kind: Cast [0-0]:
                                                            ty: float[32]
                                                            expr: Expr [62-65]:
                                                                ty: const float
                                                                kind: Lit: Float(2.2)
                                        Expr [68-78]:
                                            ty: FloatArray(Some(32), One(2))
                                            kind: Lit:     array:
                                                    Expr [69-72]:
                                                        ty: float[32]
                                                        kind: Cast [0-0]:
                                                            ty: float[32]
                                                            expr: Expr [69-72]:
                                                                ty: const float
                                                                kind: Lit: Float(3.1)
                                                    Expr [74-77]:
                                                        ty: float[32]
                                                        kind: Cast [0-0]:
                                                            ty: float[32]
                                                            expr: Expr [74-77]:
                                                                ty: const float
                                                                kind: Lit: Float(3.2)
                    Stmt [89-113]:
                        annotations: <empty>
                        kind: AssignStmt [89-113]:
                            indexed_ident: IndexedIdent [89-106]:
                                symbol_id: 8
                                name_span: [89-97]
                                index_span: [97-106]
                                indices:
                                    Expr [98-99]:
                                        ty: const int
                                        kind: Lit: Int(1)
                                    Expr [101-102]:
                                        ty: const int
                                        kind: Lit: Int(1)
                                    Expr [104-105]:
                                        ty: const int
                                        kind: Lit: Int(3)
                            rhs: Expr [109-112]:
                                ty: const float
                                kind: Lit: Float(2.3)

            [Qasm.Lowerer.TooManyIndices

              x too many indices specified
               ,-[test:3:9]
             2 |         array[float[32], 3, 2] multiDim = {{1.1, 1.2}, {2.1, 2.2}, {3.1, 3.2}};
             3 |         multiDim[1, 1, 3] = 2.3;
               :         ^^^^^^^^^^^^^^^^^
             4 |         
               `----
            ]"#]],
    );
}
