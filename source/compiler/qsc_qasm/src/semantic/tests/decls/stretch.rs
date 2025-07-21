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
                    const_value: Duration(0.0 s)
                    kind: Lit: Duration(0.0 s)
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
            duration a = 10ns;
            duration b = durationof({x [100ns] $0;});
            stretch c;
            // stretchy duration with min=100ns
            stretch d = a + 2 * c;
            // stretchy duration with backtracking by up to half b
            stretch e = -0.5 * b + c;
        "#,
        &expect![[r#"
            ClassicalDeclarationStmt [49-67]:
                symbol_id: 40
                ty_span: [49-57]
                init_expr: Expr [62-66]:
                    ty: duration
                    const_value: Duration(10.0 ns)
                    kind: Lit: Duration(10.0 ns)
            [40] Symbol [58-59]:
                name: a
                type: duration
                ty_span: [49-57]
                io_kind: Default
            ClassicalDeclarationStmt [80-121]:
                symbol_id: 41
                ty_span: [80-88]
                init_expr: Expr [93-120]:
                    ty: duration
                    const_value: Duration(100.0 ns)
                    kind: DurationofCallExpr [93-120]:
                        fn_name_span: [93-103]
                        duration: 100.0 ns
                        scope: Block [104-119]:
                            Stmt [105-118]:
                                annotations: <empty>
                                kind: GateCall [105-118]:
                                    modifiers: <empty>
                                    symbol_id: 9
                                    gate_name_span: [105-106]
                                    args: <empty>
                                    qubits:
                                        GateOperand [115-117]:
                                            kind: HardwareQubit [115-117]: 0
                                    duration: Expr [108-113]:
                                        ty: duration
                                        const_value: Duration(100.0 ns)
                                        kind: Lit: Duration(100.0 ns)
                                    classical_arity: 0
                                    quantum_arity: 1
            [41] Symbol [89-90]:
                name: b
                type: duration
                ty_span: [80-88]
                io_kind: Default
            ClassicalDeclarationStmt [134-144]:
                symbol_id: 42
                ty_span: [134-141]
                init_expr: Expr [134-144]:
                    ty: stretch
                    const_value: Duration(0.0 s)
                    kind: Lit: Duration(0.0 s)
            [42] Symbol [142-143]:
                name: c
                type: stretch
                ty_span: [134-141]
                io_kind: Default
            ClassicalDeclarationStmt [205-227]:
                symbol_id: 43
                ty_span: [205-212]
                init_expr: Expr [217-226]:
                    ty: stretch
                    const_value: Duration(10.0 ns)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [217-218]:
                            ty: duration
                            kind: SymbolId(40)
                        rhs: Expr [221-226]:
                            ty: duration
                            const_value: Duration(0.0 s)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [221-222]:
                                    ty: const int
                                    kind: Lit: Int(2)
                                rhs: Expr [225-226]:
                                    ty: stretch
                                    kind: SymbolId(42)
            [43] Symbol [213-214]:
                name: d
                type: stretch
                ty_span: [205-212]
                io_kind: Default
            ClassicalDeclarationStmt [307-332]:
                symbol_id: 44
                ty_span: [307-314]
                init_expr: Expr [319-331]:
                    ty: stretch
                    const_value: Duration(-50.0 ns)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [319-327]:
                            ty: duration
                            const_value: Duration(-50.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [320-323]:
                                    ty: const float
                                    kind: UnaryOpExpr [320-323]:
                                        op: Neg
                                        expr: Expr [320-323]:
                                            ty: const float
                                            kind: Lit: Float(0.5)
                                rhs: Expr [326-327]:
                                    ty: duration
                                    kind: SymbolId(41)
                        rhs: Expr [330-331]:
                            ty: stretch
                            kind: SymbolId(42)
            [44] Symbol [315-316]:
                name: e
                type: stretch
                ty_span: [307-314]
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
            duration start_stretch = a - .5 * durationof({x [10ns] $0;});
            duration middle_stretch = a - .5 * durationof({x [10ns] $0;}) - .5 * durationof({y [100ns] $0;});
            duration end_stretch = a - .5 * durationof({y [1000ns] $0;});
        "#,
        &expect![[r#"
            ClassicalDeclarationStmt [49-59]:
                symbol_id: 40
                ty_span: [49-56]
                init_expr: Expr [49-59]:
                    ty: stretch
                    const_value: Duration(0.0 s)
                    kind: Lit: Duration(0.0 s)
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
                    const_value: Duration(0.0 s)
                    kind: Lit: Duration(0.0 s)
            [41] Symbol [80-81]:
                name: b
                type: stretch
                ty_span: [72-79]
                io_kind: Default
            ClassicalDeclarationStmt [95-156]:
                symbol_id: 42
                ty_span: [95-103]
                init_expr: Expr [120-155]:
                    ty: duration
                    const_value: Duration(-5.0 ns)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [120-121]:
                            ty: stretch
                            kind: SymbolId(40)
                        rhs: Expr [124-155]:
                            ty: duration
                            const_value: Duration(5.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [124-126]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
                                rhs: Expr [129-155]:
                                    ty: duration
                                    kind: DurationofCallExpr [129-155]:
                                        fn_name_span: [129-139]
                                        duration: 10.0 ns
                                        scope: Block [140-154]:
                                            Stmt [141-153]:
                                                annotations: <empty>
                                                kind: GateCall [141-153]:
                                                    modifiers: <empty>
                                                    symbol_id: 9
                                                    gate_name_span: [141-142]
                                                    args: <empty>
                                                    qubits:
                                                        GateOperand [150-152]:
                                                            kind: HardwareQubit [150-152]: 0
                                                    duration: Expr [144-148]:
                                                        ty: duration
                                                        const_value: Duration(10.0 ns)
                                                        kind: Lit: Duration(10.0 ns)
                                                    classical_arity: 0
                                                    quantum_arity: 1
            [42] Symbol [104-117]:
                name: start_stretch
                type: duration
                ty_span: [95-103]
                io_kind: Default
            ClassicalDeclarationStmt [169-266]:
                symbol_id: 43
                ty_span: [169-177]
                init_expr: Expr [195-265]:
                    ty: duration
                    const_value: Duration(-55.0 ns)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [195-230]:
                            ty: duration
                            const_value: Duration(-5.0 ns)
                            kind: BinaryOpExpr:
                                op: Sub
                                lhs: Expr [195-196]:
                                    ty: stretch
                                    kind: SymbolId(40)
                                rhs: Expr [199-230]:
                                    ty: duration
                                    const_value: Duration(5.0 ns)
                                    kind: BinaryOpExpr:
                                        op: Mul
                                        lhs: Expr [199-201]:
                                            ty: const float
                                            kind: Lit: Float(0.5)
                                        rhs: Expr [204-230]:
                                            ty: duration
                                            kind: DurationofCallExpr [204-230]:
                                                fn_name_span: [204-214]
                                                duration: 10.0 ns
                                                scope: Block [215-229]:
                                                    Stmt [216-228]:
                                                        annotations: <empty>
                                                        kind: GateCall [216-228]:
                                                            modifiers: <empty>
                                                            symbol_id: 9
                                                            gate_name_span: [216-217]
                                                            args: <empty>
                                                            qubits:
                                                                GateOperand [225-227]:
                                                                    kind: HardwareQubit [225-227]: 0
                                                            duration: Expr [219-223]:
                                                                ty: duration
                                                                const_value: Duration(10.0 ns)
                                                                kind: Lit: Duration(10.0 ns)
                                                            classical_arity: 0
                                                            quantum_arity: 1
                        rhs: Expr [233-265]:
                            ty: duration
                            const_value: Duration(50.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [233-235]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
                                rhs: Expr [238-265]:
                                    ty: duration
                                    kind: DurationofCallExpr [238-265]:
                                        fn_name_span: [238-248]
                                        duration: 100.0 ns
                                        scope: Block [249-264]:
                                            Stmt [250-263]:
                                                annotations: <empty>
                                                kind: GateCall [250-263]:
                                                    modifiers: <empty>
                                                    symbol_id: 10
                                                    gate_name_span: [250-251]
                                                    args: <empty>
                                                    qubits:
                                                        GateOperand [260-262]:
                                                            kind: HardwareQubit [260-262]: 0
                                                    duration: Expr [253-258]:
                                                        ty: duration
                                                        const_value: Duration(100.0 ns)
                                                        kind: Lit: Duration(100.0 ns)
                                                    classical_arity: 0
                                                    quantum_arity: 1
            [43] Symbol [178-192]:
                name: middle_stretch
                type: duration
                ty_span: [169-177]
                io_kind: Default
            ClassicalDeclarationStmt [279-340]:
                symbol_id: 44
                ty_span: [279-287]
                init_expr: Expr [302-339]:
                    ty: duration
                    const_value: Duration(-500.0 ns)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [302-303]:
                            ty: stretch
                            kind: SymbolId(40)
                        rhs: Expr [306-339]:
                            ty: duration
                            const_value: Duration(500.0 ns)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [306-308]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
                                rhs: Expr [311-339]:
                                    ty: duration
                                    kind: DurationofCallExpr [311-339]:
                                        fn_name_span: [311-321]
                                        duration: 1000.0 ns
                                        scope: Block [322-338]:
                                            Stmt [323-337]:
                                                annotations: <empty>
                                                kind: GateCall [323-337]:
                                                    modifiers: <empty>
                                                    symbol_id: 10
                                                    gate_name_span: [323-324]
                                                    args: <empty>
                                                    qubits:
                                                        GateOperand [334-336]:
                                                            kind: HardwareQubit [334-336]: 0
                                                    duration: Expr [326-332]:
                                                        ty: duration
                                                        const_value: Duration(1000.0 ns)
                                                        kind: Lit: Duration(1000.0 ns)
                                                    classical_arity: 0
                                                    quantum_arity: 1
            [44] Symbol [288-299]:
                name: end_stretch
                type: duration
                ty_span: [279-287]
                io_kind: Default
        "#]],
    );
}
