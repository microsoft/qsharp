// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn box_can_contain_barrier() {
    check_stmt_kinds(
        r#"
            qubit q;
            box {
              barrier q;
            }
        "#,
        &expect![[r#"
            QubitDeclaration [13-21]:
                symbol_id: 8
            BoxStmt [34-78]:
                duration: <none>
                body:
                    Stmt [54-64]:
                        annotations: <empty>
                        kind: BarrierStmt [54-64]:
                            operands:
                                GateOperand [62-63]:
                                    kind: Expr [62-63]:
                                        ty: qubit
                                        kind: SymbolId(8)
        "#]],
    );
}

#[test]
#[ignore = "Duration type, stretch type, and delay are not supported yet"]
fn box_can_contain_delay() {
    check_stmt_kinds(
        r#"
            include "stdgates.inc";
            qubit q;
            duration a = 300ns;
            stretch c = 2 * a;
            box {
              delay[a] q;
            }
        "#,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [49-57]:
                        annotations: <empty>
                        kind: QubitDeclaration [49-57]:
                            symbol_id: 40
                    Stmt [70-89]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [70-89]:
                            symbol_id: 41
                            ty_span: [70-78]
                            init_expr: Expr [83-88]:
                                ty: duration
                                kind: Lit: Duration(300.0, Ns)
                    Stmt [102-120]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [102-120]:
                            symbol_id: 42
                            ty_span: [102-109]
                            init_expr: Expr [114-119]:
                                ty: const float
                                kind: BinaryOpExpr:
                                    op: Mul
                                    lhs: Expr [114-115]:
                                        ty: const float
                                        kind: Lit: Float(2.0)
                                    rhs: Expr [118-119]:
                                        ty: duration
                                        kind: SymbolId(41)
                    Stmt [133-178]:
                        annotations: <empty>
                        kind: BoxStmt [133-178]:
                            duration: <none>
                            body:
                                Stmt [153-164]:
                                    annotations: <empty>
                                    kind: Err

            [Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[test:4:13]
             3 |             qubit q;
             4 |             duration a = 300ns;
               :             ^^^^^^^^
             5 |             stretch c = 2 * a;
               `----
            , Qasm.Lowerer.NotSupported

              x stretch type values are not supported
               ,-[test:5:13]
             4 |             duration a = 300ns;
             5 |             stretch c = 2 * a;
               :             ^^^^^^^
             6 |             box {
               `----
            , Qasm.Lowerer.CannotCast

              x cannot cast expression of type duration to type const float
               ,-[test:5:29]
             4 |             duration a = 300ns;
             5 |             stretch c = 2 * a;
               :                             ^
             6 |             box {
               `----
            , Qasm.Lowerer.CannotCast

              x cannot cast expression of type const float to type stretch
               ,-[test:5:25]
             4 |             duration a = 300ns;
             5 |             stretch c = 2 * a;
               :                         ^^^^^
             6 |             box {
               `----
            , Qasm.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: delay stmt
               ,-[test:7:15]
             6 |             box {
             7 |               delay[a] q;
               :               ^^^^^^^^^^^
             8 |             }
               `----
            ]"#]],
    );
}

#[test]
fn box_can_contain_reset() {
    check_stmt_kinds(
        r#"
            include "stdgates.inc";
            qubit q;
            box {
              reset q;
            }
        "#,
        &expect![[r#"
            QubitDeclaration [49-57]:
                symbol_id: 40
            BoxStmt [70-112]:
                duration: <none>
                body:
                    Stmt [90-98]:
                        annotations: <empty>
                        kind: ResetStmt [90-98]:
                            reset_token_span: [90-95]
                            operand: GateOperand [96-97]:
                                kind: Expr [96-97]:
                                    ty: qubit
                                    kind: SymbolId(40)
        "#]],
    );
}

#[test]
fn box_can_contain_gate_call() {
    check_stmt_kinds(
        r#"
            include "stdgates.inc";
            qubit q;
            box {
              x q;
            }
        "#,
        &expect![[r#"
            QubitDeclaration [49-57]:
                symbol_id: 40
            BoxStmt [70-108]:
                duration: <none>
                body:
                    Stmt [90-94]:
                        annotations: <empty>
                        kind: GateCall [90-94]:
                            modifiers: <empty>
                            symbol_id: 9
                            gate_name_span: [90-91]
                            args: <empty>
                            qubits:
                                GateOperand [92-93]:
                                    kind: Expr [92-93]:
                                        ty: qubit
                                        kind: SymbolId(40)
                            duration: <none>
                            classical_arity: 0
                            quantum_arity: 1
        "#]],
    );
}

#[test]
fn box_can_contain_gphase() {
    check_stmt_kinds(
        r#"
            box {
              gphase(pi);
            }
        "#,
        &expect![[r#"
            BoxStmt [13-58]:
                duration: <none>
                body:
                    Stmt [33-44]:
                        annotations: <empty>
                        kind: GateCall [33-44]:
                            modifiers: <empty>
                            symbol_id: 1
                            gate_name_span: [33-39]
                            args:
                                Expr [40-42]:
                                    ty: angle
                                    kind: Cast [40-42]:
                                        ty: angle
                                        expr: Expr [40-42]:
                                            ty: const float
                                            kind: SymbolId(2)
                                        kind: Implicit
                            qubits: <empty>
                            duration: <none>
                            classical_arity: 1
                            quantum_arity: 0
        "#]],
    );
}

#[test]
fn box_can_contain_box() {
    check_stmt_kinds(
        r#"
            qubit q;
            box {
              box {
                barrier q;
              }
            }
        "#,
        &expect![[r#"
            QubitDeclaration [13-21]:
                symbol_id: 8
            BoxStmt [34-116]:
                duration: <none>
                body:
                    Stmt [54-102]:
                        annotations: <empty>
                        kind: BoxStmt [54-102]:
                            duration: <none>
                            body:
                                Stmt [76-86]:
                                    annotations: <empty>
                                    kind: BarrierStmt [76-86]:
                                        operands:
                                            GateOperand [84-85]:
                                                kind: Expr [84-85]:
                                                    ty: qubit
                                                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn with_invalid_instruction_fails() {
    check_stmt_kinds(
        "box {
        2 + 4;
    }",
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [0-26]:
                        annotations: <empty>
                        kind: BoxStmt [0-26]:
                            duration: <none>
                            body:
                                Stmt [14-20]:
                                    annotations: <empty>
                                    kind: ExprStmt [14-20]:
                                        expr: Expr [14-19]:
                                            ty: const int
                                            kind: BinaryOpExpr:
                                                op: Add
                                                lhs: Expr [14-15]:
                                                    ty: const int
                                                    kind: Lit: Int(2)
                                                rhs: Expr [18-19]:
                                                    ty: const int
                                                    kind: Lit: Int(4)

            [Qasm.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
               ,-[test:2:9]
             1 | box {
             2 |         2 + 4;
               :         ^^^^^^
             3 |     }
               `----
            ]"#]],
    );
}

#[test]
fn with_duration_fails() {
    check_stmt_kinds(
        "box [4us] { }",
        &expect![[r#"
            BoxStmt [0-13]:
                duration: Expr [5-8]:
                    ty: const duration
                    kind: Lit: Duration(4.0, Us)
                body: <empty>
        "#]],
    );
}
