// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn on_a_single_qubit() {
    check_stmt_kinds(
        "qubit q;
        delay [5ns] q;",
        &expect![[r#"
            QubitDeclaration [0-8]:
                symbol_id: 8
            DelayStmt [17-31]:
                duration: Expr [24-27]:
                    ty: const duration
                    kind: Lit: Duration(5.0, Ns)
                qubits:
                    GateOperand [29-30]:
                        kind: Expr [29-30]:
                            ty: qubit
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn on_a_single_qubit_with_variables() {
    check_stmt_kinds(
        "qubit q;
        duration stride = 5ns;
        int p = 2;
        delay [p * stride] q;",
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [0-8]:
                        annotations: <empty>
                        kind: QubitDeclaration [0-8]:
                            symbol_id: 8
                    Stmt [17-39]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [17-39]:
                            symbol_id: 9
                            ty_span: [17-25]
                            init_expr: Expr [35-38]:
                                ty: duration
                                kind: Lit: Duration(5.0, Ns)
                    Stmt [48-58]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [48-58]:
                            symbol_id: 10
                            ty_span: [48-51]
                            init_expr: Expr [56-57]:
                                ty: int
                                kind: Lit: Int(2)
                    Stmt [67-88]:
                        annotations: <empty>
                        kind: DelayStmt [67-88]:
                            duration: Expr [74-84]:
                                ty: float
                                kind: BinaryOpExpr:
                                    op: Mul
                                    lhs: Expr [74-75]:
                                        ty: float
                                        kind: Cast [0-0]:
                                            ty: float
                                            expr: Expr [74-75]:
                                                ty: int
                                                kind: SymbolId(10)
                                    rhs: Expr [78-84]:
                                        ty: duration
                                        kind: SymbolId(9)
                            qubits:
                                GateOperand [86-87]:
                                    kind: Expr [86-87]:
                                        ty: qubit
                                        kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type duration to type float
               ,-[test:4:20]
             3 |         int p = 2;
             4 |         delay [p * stride] q;
               :                    ^^^^^^
               `----
            ]"#]],
    );
}
