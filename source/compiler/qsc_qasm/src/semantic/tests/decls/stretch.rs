// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::{check_classical_decl, check_classical_decls};

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "stretch a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-10]:
                symbol_id: 8
                ty_span: [0-7]
                init_expr: Expr [0-10]:
                    ty: stretch
                    const_value: Duration(0.0 ns)
                    kind: Lit: Duration(0.0 ns)
            [8] Symbol [8-9]:
                name: a
                type: stretch
                ty_span: [0-7]
                io_kind: Default"#]],
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn spec_sample() {
    check_classical_decls(
        r#"
            include "stdgates.inc";
            duration a = 300ns;
            duration b = durationof({x $0;});
            stretch c;
            // stretchy duration with min=300ns
            stretch d = a + 2 * c;
            // stretchy duration with backtracking by up to half b
            stretch e = -0.5 * b + c;
        "#,
        &expect![[r#"
            ClassicalDeclarationStmt [49-68]:
                symbol_id: 40
                ty_span: [49-57]
                init_expr: Expr [62-67]:
                    ty: duration
                    const_value: Duration(300.0 ns)
                    kind: Lit: Duration(300.0 ns)
            [40] Symbol [58-59]:
                name: a
                type: duration
                ty_span: [49-57]
                io_kind: Default
            ClassicalDeclarationStmt [81-114]:
                symbol_id: 41
                ty_span: [81-89]
                init_expr: Expr [94-113]:
                    ty: duration
                    const_value: Duration(0.0 ns)
                    kind: DurationofCallExpr [94-113]:
                        fn_name_span: [94-104]
                        scope: Block [105-112]:
                            Stmt [106-111]:
                                annotations: <empty>
                                kind: GateCall [106-111]:
                                    modifiers: <empty>
                                    symbol_id: 9
                                    gate_name_span: [106-107]
                                    args: <empty>
                                    qubits:
                                        GateOperand [108-110]:
                                            kind: HardwareQubit [108-110]: 0
                                    duration: <none>
                                    classical_arity: 0
                                    quantum_arity: 1
            [41] Symbol [90-91]:
                name: b
                type: duration
                ty_span: [81-89]
                io_kind: Default
            ClassicalDeclarationStmt [127-137]:
                symbol_id: 42
                ty_span: [127-134]
                init_expr: Expr [127-137]:
                    ty: stretch
                    const_value: Duration(0.0 ns)
                    kind: Lit: Duration(0.0 ns)
            [42] Symbol [135-136]:
                name: c
                type: stretch
                ty_span: [127-134]
                io_kind: Default
            ClassicalDeclarationStmt [198-220]:
                symbol_id: 43
                ty_span: [198-205]
                init_expr: Expr [210-219]:
                    ty: stretch
                    const_value: Duration(300.0 ns)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [210-211]:
                            ty: duration
                            kind: SymbolId(40)
                        rhs: Expr [214-219]:
                            ty: duration
                            const_value: Duration(0.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [214-215]:
                                    ty: const int
                                    kind: Lit: Int(2)
                                rhs: Expr [218-219]:
                                    ty: stretch
                                    kind: SymbolId(42)
            [43] Symbol [206-207]:
                name: d
                type: stretch
                ty_span: [198-205]
                io_kind: Default
            ClassicalDeclarationStmt [300-325]:
                symbol_id: 44
                ty_span: [300-307]
                init_expr: Expr [312-324]:
                    ty: stretch
                    const_value: Duration(0.0 ns)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [312-320]:
                            ty: duration
                            const_value: Duration(-0.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [313-316]:
                                    ty: const float
                                    kind: UnaryOpExpr [313-316]:
                                        op: Neg
                                        expr: Expr [313-316]:
                                            ty: const float
                                            kind: Lit: Float(0.5)
                                rhs: Expr [319-320]:
                                    ty: duration
                                    kind: SymbolId(41)
                        rhs: Expr [323-324]:
                            ty: stretch
                            kind: SymbolId(42)
            [44] Symbol [308-309]:
                name: e
                type: stretch
                ty_span: [300-307]
                io_kind: Default
        "#]],
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn spec_sample_2() {
    check_classical_decls(
        r#"
            include "stdgates.inc";
            stretch a;
            stretch b;
            duration start_stretch = a - .5 * durationof({x $0;});
            duration middle_stretch = a - .5 * durationof({x $0;}) - .5 * durationof({y $0;});
            duration end_stretch = a - .5 * durationof({y $0;});
        "#,
        &expect![[r#"
            ClassicalDeclarationStmt [49-59]:
                symbol_id: 40
                ty_span: [49-56]
                init_expr: Expr [49-59]:
                    ty: stretch
                    const_value: Duration(0.0 ns)
                    kind: Lit: Duration(0.0 ns)
            [40] Symbol [57-58]:
                name: a
                type: stretch
                ty_span: [49-56]
                io_kind: Default
            ClassicalDeclarationStmt [72-82]:
                symbol_id: 41
                ty_span: [72-79]
                init_expr: Expr [72-82]:
                    ty: stretch
                    const_value: Duration(0.0 ns)
                    kind: Lit: Duration(0.0 ns)
            [41] Symbol [80-81]:
                name: b
                type: stretch
                ty_span: [72-79]
                io_kind: Default
            ClassicalDeclarationStmt [95-149]:
                symbol_id: 42
                ty_span: [95-103]
                init_expr: Expr [120-148]:
                    ty: duration
                    const_value: Duration(0.0 ns)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [120-121]:
                            ty: stretch
                            kind: SymbolId(40)
                        rhs: Expr [124-148]:
                            ty: duration
                            const_value: Duration(0.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [124-126]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
                                rhs: Expr [129-148]:
                                    ty: duration
                                    kind: DurationofCallExpr [129-148]:
                                        fn_name_span: [129-139]
                                        scope: Block [140-147]:
                                            Stmt [141-146]:
                                                annotations: <empty>
                                                kind: GateCall [141-146]:
                                                    modifiers: <empty>
                                                    symbol_id: 9
                                                    gate_name_span: [141-142]
                                                    args: <empty>
                                                    qubits:
                                                        GateOperand [143-145]:
                                                            kind: HardwareQubit [143-145]: 0
                                                    duration: <none>
                                                    classical_arity: 0
                                                    quantum_arity: 1
            [42] Symbol [104-117]:
                name: start_stretch
                type: duration
                ty_span: [95-103]
                io_kind: Default
            ClassicalDeclarationStmt [162-244]:
                symbol_id: 43
                ty_span: [162-170]
                init_expr: Expr [188-243]:
                    ty: duration
                    const_value: Duration(0.0 ns)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [188-216]:
                            ty: duration
                            const_value: Duration(0.0 ns)
                            kind: BinaryOpExpr:
                                op: Sub
                                lhs: Expr [188-189]:
                                    ty: stretch
                                    kind: SymbolId(40)
                                rhs: Expr [192-216]:
                                    ty: duration
                                    const_value: Duration(0.0 ns)
                                    kind: BinaryOpExpr:
                                        op: Mul
                                        lhs: Expr [192-194]:
                                            ty: const float
                                            kind: Lit: Float(0.5)
                                        rhs: Expr [197-216]:
                                            ty: duration
                                            kind: DurationofCallExpr [197-216]:
                                                fn_name_span: [197-207]
                                                scope: Block [208-215]:
                                                    Stmt [209-214]:
                                                        annotations: <empty>
                                                        kind: GateCall [209-214]:
                                                            modifiers: <empty>
                                                            symbol_id: 9
                                                            gate_name_span: [209-210]
                                                            args: <empty>
                                                            qubits:
                                                                GateOperand [211-213]:
                                                                    kind: HardwareQubit [211-213]: 0
                                                            duration: <none>
                                                            classical_arity: 0
                                                            quantum_arity: 1
                        rhs: Expr [219-243]:
                            ty: duration
                            const_value: Duration(0.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [219-221]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
                                rhs: Expr [224-243]:
                                    ty: duration
                                    kind: DurationofCallExpr [224-243]:
                                        fn_name_span: [224-234]
                                        scope: Block [235-242]:
                                            Stmt [236-241]:
                                                annotations: <empty>
                                                kind: GateCall [236-241]:
                                                    modifiers: <empty>
                                                    symbol_id: 10
                                                    gate_name_span: [236-237]
                                                    args: <empty>
                                                    qubits:
                                                        GateOperand [238-240]:
                                                            kind: HardwareQubit [238-240]: 0
                                                    duration: <none>
                                                    classical_arity: 0
                                                    quantum_arity: 1
            [43] Symbol [171-185]:
                name: middle_stretch
                type: duration
                ty_span: [162-170]
                io_kind: Default
            ClassicalDeclarationStmt [257-309]:
                symbol_id: 44
                ty_span: [257-265]
                init_expr: Expr [280-308]:
                    ty: duration
                    const_value: Duration(0.0 ns)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [280-281]:
                            ty: stretch
                            kind: SymbolId(40)
                        rhs: Expr [284-308]:
                            ty: duration
                            const_value: Duration(0.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [284-286]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
                                rhs: Expr [289-308]:
                                    ty: duration
                                    kind: DurationofCallExpr [289-308]:
                                        fn_name_span: [289-299]
                                        scope: Block [300-307]:
                                            Stmt [301-306]:
                                                annotations: <empty>
                                                kind: GateCall [301-306]:
                                                    modifiers: <empty>
                                                    symbol_id: 10
                                                    gate_name_span: [301-302]
                                                    args: <empty>
                                                    qubits:
                                                        GateOperand [303-305]:
                                                            kind: HardwareQubit [303-305]: 0
                                                    duration: <none>
                                                    classical_arity: 0
                                                    quantum_arity: 1
            [44] Symbol [266-277]:
                name: end_stretch
                type: duration
                ty_span: [257-265]
                io_kind: Default
        "#]],
    );
}
