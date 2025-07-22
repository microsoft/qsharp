// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

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
                                            ty: array[float[32], 2]
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
                                            ty: array[float[32], 2]
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

#[test]
fn assign_to_mutable_static_array_ref() {
    let source = "
        def f(mutable array[bool, 2, 3] a) {
            a[0][0] = true;
        }
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            DefStmt [9-83]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                return_type_span: [0-0]
                body: Block [44-83]:
                    Stmt [58-73]:
                        annotations: <empty>
                        kind: AssignStmt [58-73]:
                            lhs: Expr [58-64]:
                                ty: bool
                                kind: IndexedExpr [58-64]:
                                    collection: Expr [58-61]:
                                        ty: mutable array[bool, 3]
                                        kind: IndexedExpr [58-61]:
                                            collection: Expr [58-59]:
                                                ty: mutable array[bool, 2, 3]
                                                kind: SymbolId(9)
                                            index: Expr [60-61]:
                                                ty: const int
                                                kind: Lit: Int(0)
                                    index: Expr [63-64]:
                                        ty: const int
                                        kind: Lit: Int(0)
                            rhs: Expr [68-72]:
                                ty: bool
                                kind: Lit: Bool(true)
        "#]],
    );
}

#[test]
fn assign_to_mutable_dyn_array_ref() {
    let source = "
        def f(mutable array[bool, #dim = 2] a) {
            a[0][0] = true;
        }
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            DefStmt [9-87]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                return_type_span: [0-0]
                body: Block [48-87]:
                    Stmt [62-77]:
                        annotations: <empty>
                        kind: AssignStmt [62-77]:
                            lhs: Expr [62-68]:
                                ty: bool
                                kind: IndexedExpr [62-68]:
                                    collection: Expr [62-65]:
                                        ty: mutable array[bool, #dim = 1]
                                        kind: IndexedExpr [62-65]:
                                            collection: Expr [62-63]:
                                                ty: mutable array[bool, #dim = 2]
                                                kind: SymbolId(9)
                                            index: Expr [64-65]:
                                                ty: const int
                                                kind: Lit: Int(0)
                                    index: Expr [67-68]:
                                        ty: const int
                                        kind: Lit: Int(0)
                            rhs: Expr [72-76]:
                                ty: bool
                                kind: Lit: Bool(true)
        "#]],
    );
}

#[test]
fn assign_to_readonly_static_array_ref_errors() {
    let source = "
        def f(readonly array[bool, 2, 3] a) {
            a[0][0] = true;
        }
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-84]:
                        annotations: <empty>
                        kind: DefStmt [9-84]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                            return_type_span: [0-0]
                            body: Block [45-84]:
                                Stmt [59-74]:
                                    annotations: <empty>
                                    kind: AssignStmt [59-74]:
                                        lhs: Expr [59-66]:
                                            ty: unknown
                                            kind: Err
                                        rhs: Expr [59-74]:
                                            ty: unknown
                                            kind: Err

            [Qasm.Lowerer.CannotUpdateReadonlyArrayRef

              x cannot update readonly array reference a
               ,-[test:3:13]
             2 |         def f(readonly array[bool, 2, 3] a) {
             3 |             a[0][0] = true;
               :             ^
             4 |         }
               `----
              help: mutable array references must be declared with the keyword `mutable`
            ]"#]],
    );
}

#[test]
fn assign_to_readonly_dyn_array_ref_errors() {
    let source = "
        def f(readonly array[bool, #dim = 2] a) {
            a[0][0] = true;
        }
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-88]:
                        annotations: <empty>
                        kind: DefStmt [9-88]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                            return_type_span: [0-0]
                            body: Block [49-88]:
                                Stmt [63-78]:
                                    annotations: <empty>
                                    kind: AssignStmt [63-78]:
                                        lhs: Expr [63-70]:
                                            ty: unknown
                                            kind: Err
                                        rhs: Expr [63-78]:
                                            ty: unknown
                                            kind: Err

            [Qasm.Lowerer.CannotUpdateReadonlyArrayRef

              x cannot update readonly array reference a
               ,-[test:3:13]
             2 |         def f(readonly array[bool, #dim = 2] a) {
             3 |             a[0][0] = true;
               :             ^
             4 |         }
               `----
              help: mutable array references must be declared with the keyword `mutable`
            ]"#]],
    );
}
