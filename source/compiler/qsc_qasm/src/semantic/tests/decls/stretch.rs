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
// todo: durationof
fn spec_sample() {
    check_classical_decls(
        r#"
            duration a = 300ns;
            //duration b = durationof({x $0;});
            stretch c;
            // stretchy duration with min=300ns
            stretch d = a + 2 * c;
            // stretchy duration with backtracking by up to half b
            stretch e = -0.5 * b + c;
        "#,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [13-32]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [13-32]:
                            symbol_id: 8
                            ty_span: [13-21]
                            init_expr: Expr [26-31]:
                                ty: duration
                                const_value: Duration(300.0 ns)
                                kind: Lit: Duration(300.0 ns)
                    Stmt [93-103]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [93-103]:
                            symbol_id: 9
                            ty_span: [93-100]
                            init_expr: Expr [93-103]:
                                ty: stretch
                                const_value: Duration(0.0 ns)
                                kind: Lit: Duration(0.0 ns)
                    Stmt [164-186]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [164-186]:
                            symbol_id: 10
                            ty_span: [164-171]
                            init_expr: Expr [176-185]:
                                ty: duration
                                const_value: Duration(300.0 ns)
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [176-177]:
                                        ty: duration
                                        kind: SymbolId(8)
                                    rhs: Expr [180-185]:
                                        ty: duration
                                        const_value: Duration(0.0 ns)
                                        kind: BinaryOpExpr:
                                            op: Mul
                                            lhs: Expr [180-181]:
                                                ty: const int
                                                kind: Lit: Int(2)
                                            rhs: Expr [184-185]:
                                                ty: stretch
                                                kind: SymbolId(9)
                    Stmt [266-291]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [266-291]:
                            symbol_id: 12
                            ty_span: [266-273]
                            init_expr: Expr [278-290]:
                                ty: duration
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [278-286]:
                                        ty: const float
                                        kind: BinaryOpExpr:
                                            op: Mul
                                            lhs: Expr [279-282]:
                                                ty: const float
                                                kind: UnaryOpExpr [279-282]:
                                                    op: Neg
                                                    expr: Expr [279-282]:
                                                        ty: const float
                                                        kind: Lit: Float(0.5)
                                            rhs: Expr [285-286]:
                                                ty: unknown
                                                kind: SymbolId(11)
                                    rhs: Expr [289-290]:
                                        ty: stretch
                                        kind: SymbolId(9)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type duration to type stretch
               ,-[test:6:25]
             5 |             // stretchy duration with min=300ns
             6 |             stretch d = a + 2 * c;
               :                         ^^^^^^^^^
             7 |             // stretchy duration with backtracking by up to half b
               `----
            , Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: b
               ,-[test:8:32]
             7 |             // stretchy duration with backtracking by up to half b
             8 |             stretch e = -0.5 * b + c;
               :                                ^
             9 |         
               `----
            , Qasm.Lowerer.DurationMustBeKnownAtCompileTime

              x duration must be known at compile time
               ,-[test:8:25]
             7 |             // stretchy duration with backtracking by up to half b
             8 |             stretch e = -0.5 * b + c;
               :                         ^^^^^^^^^^^^
             9 |         
               `----
            , Qasm.Lowerer.CannotCast

              x cannot cast expression of type duration to type stretch
               ,-[test:8:25]
             7 |             // stretchy duration with backtracking by up to half b
             8 |             stretch e = -0.5 * b + c;
               :                         ^^^^^^^^^^^^
             9 |         
               `----
            ]"#]],
    );
}

#[test]
// todo: durationof
fn spec_sample_2() {
    check_classical_decls(
        r#"
            stretch a;
            stretch b;
            duration start_stretch = a - .5 * durationof({x $0;});
            duration middle_stretch = a - .5 * durationof({x $0;}) - .5 * durationof({y $0;});
            duration end_stretch = a - .5 * durationof({y $0;});
        "#,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [13-32]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [13-32]:
                            symbol_id: 8
                            ty_span: [13-21]
                            init_expr: Expr [26-31]:
                                ty: duration
                                const_value: Duration(300.0 ns)
                                kind: Lit: Duration(300.0 ns)
                    Stmt [93-103]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [93-103]:
                            symbol_id: 9
                            ty_span: [93-100]
                            init_expr: Expr [93-103]:
                                ty: stretch
                                const_value: Duration(0.0 ns)
                                kind: Lit: Duration(0.0 ns)
                    Stmt [164-186]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [164-186]:
                            symbol_id: 10
                            ty_span: [164-171]
                            init_expr: Expr [176-185]:
                                ty: duration
                                const_value: Duration(300.0 ns)
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [176-177]:
                                        ty: duration
                                        kind: SymbolId(8)
                                    rhs: Expr [180-185]:
                                        ty: duration
                                        const_value: Duration(0.0 ns)
                                        kind: BinaryOpExpr:
                                            op: Mul
                                            lhs: Expr [180-181]:
                                                ty: const int
                                                kind: Lit: Int(2)
                                            rhs: Expr [184-185]:
                                                ty: stretch
                                                kind: SymbolId(9)
                    Stmt [266-291]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [266-291]:
                            symbol_id: 12
                            ty_span: [266-273]
                            init_expr: Expr [278-290]:
                                ty: duration
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [278-286]:
                                        ty: const float
                                        kind: BinaryOpExpr:
                                            op: Mul
                                            lhs: Expr [279-282]:
                                                ty: const float
                                                kind: UnaryOpExpr [279-282]:
                                                    op: Neg
                                                    expr: Expr [279-282]:
                                                        ty: const float
                                                        kind: Lit: Float(0.5)
                                            rhs: Expr [285-286]:
                                                ty: unknown
                                                kind: SymbolId(11)
                                    rhs: Expr [289-290]:
                                        ty: stretch
                                        kind: SymbolId(9)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type duration to type stretch
               ,-[test:6:25]
             5 |             // stretchy duration with min=300ns
             6 |             stretch d = a + 2 * c;
               :                         ^^^^^^^^^
             7 |             // stretchy duration with backtracking by up to half b
               `----
            , Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: b
               ,-[test:8:32]
             7 |             // stretchy duration with backtracking by up to half b
             8 |             stretch e = -0.5 * b + c;
               :                                ^
             9 |         
               `----
            , Qasm.Lowerer.DurationMustBeKnownAtCompileTime

              x duration must be known at compile time
               ,-[test:8:25]
             7 |             // stretchy duration with backtracking by up to half b
             8 |             stretch e = -0.5 * b + c;
               :                         ^^^^^^^^^^^^
             9 |         
               `----
            , Qasm.Lowerer.CannotCast

              x cannot cast expression of type duration to type stretch
               ,-[test:8:25]
             7 |             // stretchy duration with backtracking by up to half b
             8 |             stretch e = -0.5 * b + c;
               :                         ^^^^^^^^^^^^
             9 |         
               `----
            ]"#]],
    );
}
