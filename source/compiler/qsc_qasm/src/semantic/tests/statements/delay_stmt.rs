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
                    ty: duration
                    const_value: Duration(5.0 ns)
                    kind: Lit: Duration(5.0 ns)
                qubits:
                    GateOperand [29-30]:
                        kind: Expr [29-30]:
                            ty: qubit
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn cannot_have_a_negative_duration() {
    check_stmt_kinds(
        "qubit q;
        duration d = 1s - 4s;
        delay [d] q;",
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [0-8]:
                        annotations: <empty>
                        kind: QubitDeclaration [0-8]:
                            symbol_id: 8
                    Stmt [17-38]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [17-38]:
                            symbol_id: 9
                            ty_span: [17-25]
                            init_expr: Expr [30-37]:
                                ty: duration
                                const_value: Duration(-3.0 s)
                                kind: BinaryOpExpr:
                                    op: Sub
                                    lhs: Expr [30-32]:
                                        ty: duration
                                        kind: Lit: Duration(1.0 s)
                                    rhs: Expr [35-37]:
                                        ty: duration
                                        kind: Lit: Duration(4.0 s)
                    Stmt [47-59]:
                        annotations: <empty>
                        kind: DelayStmt [47-59]:
                            duration: Expr [54-55]:
                                ty: duration
                                const_value: Duration(-3.0 s)
                                kind: SymbolId(9)
                            qubits:
                                GateOperand [57-58]:
                                    kind: Expr [57-58]:
                                        ty: qubit
                                        kind: SymbolId(8)

            [Qasm.Lowerer.DesignatorMustBePositiveDuration

              x designator must be a positive duration
               ,-[test:3:16]
             2 |         duration d = 1s - 4s;
             3 |         delay [d] q;
               :                ^
               `----
            ]"#]],
    );
}

#[test]
fn on_a_single_qubit_with_variables() {
    check_stmt_kinds(
        "qubit q;
        duration stride = 5ns;
        const int p = 2;
        delay [p * stride] q;",
        &expect![[r#"
            QubitDeclaration [0-8]:
                symbol_id: 8
            ClassicalDeclarationStmt [17-39]:
                symbol_id: 9
                ty_span: [17-25]
                init_expr: Expr [35-38]:
                    ty: duration
                    const_value: Duration(5.0 ns)
                    kind: Lit: Duration(5.0 ns)
            ClassicalDeclarationStmt [48-64]:
                symbol_id: 10
                ty_span: [54-57]
                init_expr: Expr [62-63]:
                    ty: const int
                    const_value: Int(2)
                    kind: Lit: Int(2)
            DelayStmt [73-94]:
                duration: Expr [80-90]:
                    ty: duration
                    const_value: Duration(10.0 ns)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [80-81]:
                            ty: const int
                            kind: SymbolId(10)
                        rhs: Expr [84-90]:
                            ty: duration
                            kind: SymbolId(9)
                qubits:
                    GateOperand [92-93]:
                        kind: Expr [92-93]:
                            ty: qubit
                            kind: SymbolId(8)
        "#]],
    );
}
