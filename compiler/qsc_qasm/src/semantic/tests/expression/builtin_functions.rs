// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn builtin_call_with_invalid_input_types_fails() {
    let source = "
        mod(9, true);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-22]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

              x There is no valid overload of `mod` for inputs: (const int, const bool)
              | Overloads available are:
              |     def (const int, const int) -> const int
              |     def (const float, const float) -> const float
               ,-[test:2:9]
             1 | 
             2 |         mod(9, true);
               :         ^^^^^^^^^^^^
             3 |     
               `----
            ]"#]],
    );
}

#[test]
fn builtin_call_with_zero_arguments_fails() {
    let source = "
        mod();
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-15]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

              x There is no valid overload of `mod` for inputs: ()
              | Overloads available are:
              |     def (const int, const int) -> const int
              |     def (const float, const float) -> const float
               ,-[test:2:9]
             1 | 
             2 |         mod();
               :         ^^^^^
             3 |     
               `----
            ]"#]],
    );
}

#[test]
fn builtin_call_with_lower_arity_fails() {
    let source = "
        mod(9);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-16]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

              x There is no valid overload of `mod` for inputs: (const int)
              | Overloads available are:
              |     def (const int, const int) -> const int
              |     def (const float, const float) -> const float
               ,-[test:2:9]
             1 | 
             2 |         mod(9);
               :         ^^^^^^
             3 |     
               `----
            ]"#]],
    );
}

#[test]
fn builtin_call_with_higher_arity_fails() {
    let source = "
        mod(9, 7, 2);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-22]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

              x There is no valid overload of `mod` for inputs: (const int, const int,
              | const int)
              | Overloads available are:
              |     def (const int, const int) -> const int
              |     def (const float, const float) -> const float
               ,-[test:2:9]
             1 | 
             2 |         mod(9, 7, 2);
               :         ^^^^^^^^^^^^
             3 |     
               `----
            ]"#]],
    );
}

#[test]
fn builtin_call_with_const_expr_succeeds() {
    let source = "
        const int a = 9;
        const int b = 7;
        mod(a + b, b);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-25]:
                symbol_id: 8
                ty_span: [15-18]
                init_expr: Expr [23-24]:
                    ty: const int
                    const_value: Int(9)
                    kind: Lit: Int(9)
            ClassicalDeclarationStmt [34-50]:
                symbol_id: 9
                ty_span: [40-43]
                init_expr: Expr [48-49]:
                    ty: const int
                    const_value: Int(7)
                    kind: Lit: Int(7)
            ExprStmt [59-73]:
                expr: Expr [59-72]:
                    ty: const int
                    const_value: Int(2)
                    kind: BuiltinFunctionCall [59-72]:
                        fn_name_span: [59-62]
                        name: mod
                        function_ty: def (const int, const int) -> const int
                        args:
                            Expr [63-68]:
                                ty: const int
                                const_value: Int(16)
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [63-64]:
                                        ty: const int
                                        kind: SymbolId(8)
                                    rhs: Expr [67-68]:
                                        ty: const int
                                        kind: SymbolId(9)
                            Expr [70-71]:
                                ty: const int
                                const_value: Int(7)
                                kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn nested_builtin_call_succeeds() {
    let source = "
        const int a = 9;
        const int b = 7;
        mod(a, mod(a, b));
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-25]:
                symbol_id: 8
                ty_span: [15-18]
                init_expr: Expr [23-24]:
                    ty: const int
                    const_value: Int(9)
                    kind: Lit: Int(9)
            ClassicalDeclarationStmt [34-50]:
                symbol_id: 9
                ty_span: [40-43]
                init_expr: Expr [48-49]:
                    ty: const int
                    const_value: Int(7)
                    kind: Lit: Int(7)
            ExprStmt [59-77]:
                expr: Expr [59-76]:
                    ty: const int
                    const_value: Int(1)
                    kind: BuiltinFunctionCall [59-76]:
                        fn_name_span: [59-62]
                        name: mod
                        function_ty: def (const int, const int) -> const int
                        args:
                            Expr [63-64]:
                                ty: const int
                                const_value: Int(9)
                                kind: SymbolId(8)
                            Expr [66-75]:
                                ty: const int
                                const_value: Int(2)
                                kind: BuiltinFunctionCall [66-75]:
                                    fn_name_span: [66-69]
                                    name: mod
                                    function_ty: def (const int, const int) -> const int
                                    args:
                                        Expr [70-71]:
                                            ty: const int
                                            const_value: Int(9)
                                            kind: SymbolId(8)
                                        Expr [73-74]:
                                            ty: const int
                                            const_value: Int(7)
                                            kind: SymbolId(9)
        "#]],
    );
}

// ----------------------------------
// arccos

#[test]
fn arccos() {
    let source = "
        arccos(0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(1.0471975511965979)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-15]
                    name: arccos
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-19]:
                            ty: const float
                            const_value: Float(0.5)
                            kind: Lit: Float(0.5)
    "#]],
    );
}

#[test]
fn arccos_negative_domain_edge_case() {
    let source = "
        arccos(-1.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(3.141592653589793)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-15]
                    name: arccos
                    function_ty: def (const float) -> const float
                    args:
                        Expr [17-19]:
                            ty: const float
                            const_value: Float(-1.0)
                            kind: UnaryOpExpr [17-19]:
                                op: Neg
                                expr: Expr [17-19]:
                                    ty: const float
                                    kind: Lit: Float(1.0)
    "#]],
    );
}

#[test]
fn arccos_positive_domain_edge_case() {
    let source = "
        arccos(1.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-20]:
            expr: Expr [9-19]:
                ty: const float
                const_value: Float(0.0)
                kind: BuiltinFunctionCall [9-19]:
                    fn_name_span: [9-15]
                    name: arccos
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-18]:
                            ty: const float
                            const_value: Float(1.0)
                            kind: Lit: Float(1.0)
    "#]],
    );
}

#[test]
fn arccos_negative_domain_error() {
    let source = "
        arccos(-1.01);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-23]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.DomainError

          x arccos input should be in the range [-1.0, 1.0]
           ,-[test:2:9]
         1 | 
         2 |         arccos(-1.01);
           :         ^^^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

#[test]
fn arccos_positive_domain_error() {
    let source = "
        arccos(1.01);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-22]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.DomainError

          x arccos input should be in the range [-1.0, 1.0]
           ,-[test:2:9]
         1 | 
         2 |         arccos(1.01);
           :         ^^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

// ----------------------------------
// arcsin

#[test]
fn arcsin() {
    let source = "
        arcsin(0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(0.5235987755982989)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-15]
                    name: arcsin
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-19]:
                            ty: const float
                            const_value: Float(0.5)
                            kind: Lit: Float(0.5)
    "#]],
    );
}

#[test]
fn arcsin_negative_domain_edge_case() {
    let source = "
        arcsin(-1.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(-1.5707963267948966)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-15]
                    name: arcsin
                    function_ty: def (const float) -> const float
                    args:
                        Expr [17-19]:
                            ty: const float
                            const_value: Float(-1.0)
                            kind: UnaryOpExpr [17-19]:
                                op: Neg
                                expr: Expr [17-19]:
                                    ty: const float
                                    kind: Lit: Float(1.0)
    "#]],
    );
}

#[test]
fn arcsin_positive_domain_edge_case() {
    let source = "
        arcsin(1.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-20]:
            expr: Expr [9-19]:
                ty: const float
                const_value: Float(1.5707963267948966)
                kind: BuiltinFunctionCall [9-19]:
                    fn_name_span: [9-15]
                    name: arcsin
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-18]:
                            ty: const float
                            const_value: Float(1.0)
                            kind: Lit: Float(1.0)
    "#]],
    );
}

#[test]
fn arcsin_negative_domain_error() {
    let source = "
        arcsin(-1.01);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-23]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.DomainError

          x arcsin input should be in the range [-1.0, 1.0]
           ,-[test:2:9]
         1 | 
         2 |         arcsin(-1.01);
           :         ^^^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

#[test]
fn arcsin_positive_domain_error() {
    let source = "
        arcsin(1.01);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-22]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.DomainError

          x arcsin input should be in the range [-1.0, 1.0]
           ,-[test:2:9]
         1 | 
         2 |         arcsin(1.01);
           :         ^^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

// ----------------------------------
// arctan

#[test]
fn arctan() {
    let source = "
        arctan(0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(0.4636476090008061)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-15]
                    name: arctan
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-19]:
                            ty: const float
                            const_value: Float(0.5)
                            kind: Lit: Float(0.5)
    "#]],
    );
}

// ----------------------------------
// ceiling

#[test]
fn ceiling_positive() {
    let source = "
        ceiling(0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-22]:
            expr: Expr [9-21]:
                ty: const float
                const_value: Float(1.0)
                kind: BuiltinFunctionCall [9-21]:
                    fn_name_span: [9-16]
                    name: ceiling
                    function_ty: def (const float) -> const float
                    args:
                        Expr [17-20]:
                            ty: const float
                            const_value: Float(0.5)
                            kind: Lit: Float(0.5)
    "#]],
    );
}

#[test]
fn ceiling_positive_edge_case() {
    let source = "
        ceiling(1.0);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-22]:
            expr: Expr [9-21]:
                ty: const float
                const_value: Float(1.0)
                kind: BuiltinFunctionCall [9-21]:
                    fn_name_span: [9-16]
                    name: ceiling
                    function_ty: def (const float) -> const float
                    args:
                        Expr [17-20]:
                            ty: const float
                            const_value: Float(1.0)
                            kind: Lit: Float(1.0)
    "#]],
    );
}

#[test]
fn ceiling_negative() {
    let source = "
        ceiling(-0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-23]:
            expr: Expr [9-22]:
                ty: const float
                const_value: Float(-0.0)
                kind: BuiltinFunctionCall [9-22]:
                    fn_name_span: [9-16]
                    name: ceiling
                    function_ty: def (const float) -> const float
                    args:
                        Expr [18-21]:
                            ty: const float
                            const_value: Float(-0.5)
                            kind: UnaryOpExpr [18-21]:
                                op: Neg
                                expr: Expr [18-21]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
    "#]],
    );
}

#[test]
fn ceiling_negative_edge_case() {
    let source = "
        ceiling(-1.0);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-23]:
            expr: Expr [9-22]:
                ty: const float
                const_value: Float(-1.0)
                kind: BuiltinFunctionCall [9-22]:
                    fn_name_span: [9-16]
                    name: ceiling
                    function_ty: def (const float) -> const float
                    args:
                        Expr [18-21]:
                            ty: const float
                            const_value: Float(-1.0)
                            kind: UnaryOpExpr [18-21]:
                                op: Neg
                                expr: Expr [18-21]:
                                    ty: const float
                                    kind: Lit: Float(1.0)
    "#]],
    );
}

// ----------------------------------
// cos

#[test]
fn cos_float() {
    let source = "
        cos(pi);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-17]:
            expr: Expr [9-16]:
                ty: const float
                const_value: Float(-1.0)
                kind: BuiltinFunctionCall [9-16]:
                    fn_name_span: [9-12]
                    name: cos
                    function_ty: def (const float) -> const float
                    args:
                        Expr [13-15]:
                            ty: const float
                            const_value: Float(3.141592653589793)
                            kind: SymbolId(2)
    "#]],
    );
}

#[test]
fn cos_angle() {
    let source = "
        const angle a = pi;
        cos(a);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-28]:
            symbol_id: 8
            ty_span: [15-20]
            init_expr: Expr [25-27]:
                ty: const angle
                const_value: Angle(3.141592653589793)
                kind: Cast [0-0]:
                    ty: const angle
                    expr: Expr [25-27]:
                        ty: const float
                        kind: SymbolId(2)
        ExprStmt [37-44]:
            expr: Expr [37-43]:
                ty: const float
                const_value: Float(-1.0)
                kind: BuiltinFunctionCall [37-43]:
                    fn_name_span: [37-40]
                    name: cos
                    function_ty: def (const angle) -> const float
                    args:
                        Expr [41-42]:
                            ty: const angle
                            const_value: Angle(3.141592653589793)
                            kind: SymbolId(8)
    "#]],
    );
}

// ----------------------------------
// exp

#[test]
fn exp_float() {
    let source = "
        exp(2.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-17]:
            expr: Expr [9-16]:
                ty: const float
                const_value: Float(7.38905609893065)
                kind: BuiltinFunctionCall [9-16]:
                    fn_name_span: [9-12]
                    name: exp
                    function_ty: def (const float) -> const float
                    args:
                        Expr [13-15]:
                            ty: const float
                            const_value: Float(2.0)
                            kind: Lit: Float(2.0)
    "#]],
    );
}

#[test]
fn exp_complex() {
    let source = "
        const complex a = 2 + 3 im;
        exp(a);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-36]:
                symbol_id: 8
                ty_span: [15-22]
                init_expr: Expr [27-35]:
                    ty: const complex[float]
                    const_value: Complex(2.0, 3.0)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [27-28]:
                            ty: const complex[float]
                            kind: Lit: Complex(2.0, 0.0)
                        rhs: Expr [31-35]:
                            ty: const complex[float]
                            kind: Lit: Complex(0.0, 3.0)
            ExprStmt [45-52]:
                expr: Expr [45-51]:
                    ty: const complex[float]
                    const_value: Complex(-7.315110094901102, 1.0427436562359043)
                    kind: BuiltinFunctionCall [45-51]:
                        fn_name_span: [45-48]
                        name: exp
                        function_ty: def (const complex[float]) -> const complex[float]
                        args:
                            Expr [49-50]:
                                ty: const complex[float]
                                const_value: Complex(2.0, 3.0)
                                kind: SymbolId(8)
        "#]],
    );
}

// ----------------------------------
// floor

#[test]
fn floor_positive() {
    let source = "
        floor(0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-20]:
            expr: Expr [9-19]:
                ty: const float
                const_value: Float(0.0)
                kind: BuiltinFunctionCall [9-19]:
                    fn_name_span: [9-14]
                    name: floor
                    function_ty: def (const float) -> const float
                    args:
                        Expr [15-18]:
                            ty: const float
                            const_value: Float(0.5)
                            kind: Lit: Float(0.5)
    "#]],
    );
}

#[test]
fn floor_positive_edge_case() {
    let source = "
        floor(1.0);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-20]:
            expr: Expr [9-19]:
                ty: const float
                const_value: Float(1.0)
                kind: BuiltinFunctionCall [9-19]:
                    fn_name_span: [9-14]
                    name: floor
                    function_ty: def (const float) -> const float
                    args:
                        Expr [15-18]:
                            ty: const float
                            const_value: Float(1.0)
                            kind: Lit: Float(1.0)
    "#]],
    );
}

#[test]
fn floor_negative() {
    let source = "
        floor(-0.5);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(-1.0)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-14]
                    name: floor
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-19]:
                            ty: const float
                            const_value: Float(-0.5)
                            kind: UnaryOpExpr [16-19]:
                                op: Neg
                                expr: Expr [16-19]:
                                    ty: const float
                                    kind: Lit: Float(0.5)
    "#]],
    );
}

#[test]
fn floor_negative_edge_case() {
    let source = "
        floor(-1.0);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-21]:
            expr: Expr [9-20]:
                ty: const float
                const_value: Float(-1.0)
                kind: BuiltinFunctionCall [9-20]:
                    fn_name_span: [9-14]
                    name: floor
                    function_ty: def (const float) -> const float
                    args:
                        Expr [16-19]:
                            ty: const float
                            const_value: Float(-1.0)
                            kind: UnaryOpExpr [16-19]:
                                op: Neg
                                expr: Expr [16-19]:
                                    ty: const float
                                    kind: Lit: Float(1.0)
    "#]],
    );
}

// ----------------------------------
// log

#[test]
fn log() {
    let source = "
        log(2.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-17]:
            expr: Expr [9-16]:
                ty: const float
                const_value: Float(0.6931471805599453)
                kind: BuiltinFunctionCall [9-16]:
                    fn_name_span: [9-12]
                    name: log
                    function_ty: def (const float) -> const float
                    args:
                        Expr [13-15]:
                            ty: const float
                            const_value: Float(2.0)
                            kind: Lit: Float(2.0)
    "#]],
    );
}

// ----------------------------------
// mod

#[test]
fn mod_int() {
    let source = "
        mod(9, 7);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ExprStmt [9-19]:
                expr: Expr [9-18]:
                    ty: const int
                    const_value: Int(2)
                    kind: BuiltinFunctionCall [9-18]:
                        fn_name_span: [9-12]
                        name: mod
                        function_ty: def (const int, const int) -> const int
                        args:
                            Expr [13-14]:
                                ty: const int
                                const_value: Int(9)
                                kind: Lit: Int(9)
                            Expr [16-17]:
                                ty: const int
                                const_value: Int(7)
                                kind: Lit: Int(7)
        "#]],
    );
}

#[test]
fn mod_int_divide_by_zero_error() {
    let source = "
        mod(9, 0);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-19]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.DivisionByZero

              x division by zero error during const evaluation
               ,-[test:2:9]
             1 | 
             2 |         mod(9, 0);
               :         ^^^^^^^^^
             3 |     
               `----
            ]"#]],
    );
}

#[test]
fn mod_float() {
    let source = "
        mod(9, 7.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ExprStmt [9-20]:
                expr: Expr [9-19]:
                    ty: const float
                    const_value: Float(2.0)
                    kind: BuiltinFunctionCall [9-19]:
                        fn_name_span: [9-12]
                        name: mod
                        function_ty: def (const float, const float) -> const float
                        args:
                            Expr [13-14]:
                                ty: const int
                                const_value: Int(9)
                                kind: Lit: Int(9)
                            Expr [16-18]:
                                ty: const float
                                const_value: Float(7.0)
                                kind: Lit: Float(7.0)
        "#]],
    );
}

#[test]
fn mod_float_divide_by_zero_error() {
    let source = "
        mod(9., 0.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-21]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.DivisionByZero

              x division by zero error during const evaluation
               ,-[test:2:9]
             1 | 
             2 |         mod(9., 0.);
               :         ^^^^^^^^^^^
             3 |     
               `----
            ]"#]],
    );
}

// ----------------------------------
// popcount

#[test]
fn popcount() {
    let source = r#"
        const bit[5] a = "10101";
        popcount(a);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-34]:
                symbol_id: 8
                ty_span: [15-21]
                init_expr: Expr [26-33]:
                    ty: const bit[5]
                    const_value: Bitstring("10101")
                    kind: Lit: Bitstring("10101")
            ExprStmt [43-55]:
                expr: Expr [43-54]:
                    ty: const uint
                    const_value: Int(3)
                    kind: BuiltinFunctionCall [43-54]:
                        fn_name_span: [43-51]
                        name: popcount
                        function_ty: def (const bit[5]) -> const uint
                        args:
                            Expr [52-53]:
                                ty: const bit[5]
                                const_value: Bitstring("10101")
                                kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn popcount_literal() {
    let source = r#"
        popcount("10101");
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ExprStmt [9-27]:
                expr: Expr [9-26]:
                    ty: const uint
                    const_value: Int(3)
                    kind: BuiltinFunctionCall [9-26]:
                        fn_name_span: [9-17]
                        name: popcount
                        function_ty: def (const bit[5]) -> const uint
                        args:
                            Expr [18-25]:
                                ty: const bit[5]
                                const_value: Bitstring("10101")
                                kind: Lit: Bitstring("10101")
        "#]],
    );
}

#[test]
fn popcount_unsized_type_error() {
    let source = r#"
        popcount(2);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-21]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

          x There is no valid overload of `popcount` for inputs: (const int)
          | Overloads available are:
          |     fn (bit[n]) -> uint
           ,-[test:2:9]
         1 | 
         2 |         popcount(2);
           :         ^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

// ----------------------------------
// pow

#[test]
#[ignore = "pow builtin collides with the pow gate modifier"]
fn pow_int() {
    let source = r"
        pow(2, 3);
    ";

    check_stmt_kinds(source, &expect![[r#""#]]);
}

#[test]
#[ignore = "pow builtin collides with pow gate modifier"]
fn pow_float() {
    let source = r"
        pow(2., 3.);
    ";

    check_stmt_kinds(source, &expect![[r#""#]]);
}

#[test]
#[ignore = "pow builtin collides with pow gate modifier"]
fn pow_complex() {
    let source = r"
        pow(2 im, 3);
    ";

    check_stmt_kinds(source, &expect![[r#""#]]);
}

// ----------------------------------
// rotl

#[test]
fn rotl_bitarray() {
    let source = r#"
        const bit[5] a = "10001";
        rotl(a, 1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-34]:
                symbol_id: 8
                ty_span: [15-21]
                init_expr: Expr [26-33]:
                    ty: const bit[5]
                    const_value: Bitstring("10001")
                    kind: Lit: Bitstring("10001")
            ExprStmt [43-54]:
                expr: Expr [43-53]:
                    ty: const bit[5]
                    const_value: Bitstring("00011")
                    kind: BuiltinFunctionCall [43-53]:
                        fn_name_span: [43-47]
                        name: rotl
                        function_ty: def (const bit[5], const int) -> const bit[5]
                        args:
                            Expr [48-49]:
                                ty: const bit[5]
                                const_value: Bitstring("10001")
                                kind: SymbolId(8)
                            Expr [51-52]:
                                ty: const int
                                const_value: Int(1)
                                kind: Lit: Int(1)
        "#]],
    );
}

#[test]
fn rotl_bitarray_negative_distance() {
    let source = r#"
        const bit[5] a = "10001";
        rotl(a, -1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-34]:
                symbol_id: 8
                ty_span: [15-21]
                init_expr: Expr [26-33]:
                    ty: const bit[5]
                    const_value: Bitstring("10001")
                    kind: Lit: Bitstring("10001")
            ExprStmt [43-55]:
                expr: Expr [43-54]:
                    ty: const bit[5]
                    const_value: Bitstring("11000")
                    kind: BuiltinFunctionCall [43-54]:
                        fn_name_span: [43-47]
                        name: rotl
                        function_ty: def (const bit[5], const int) -> const bit[5]
                        args:
                            Expr [48-49]:
                                ty: const bit[5]
                                const_value: Bitstring("10001")
                                kind: SymbolId(8)
                            Expr [52-53]:
                                ty: const int
                                const_value: Int(-1)
                                kind: UnaryOpExpr [52-53]:
                                    op: Neg
                                    expr: Expr [52-53]:
                                        ty: const int
                                        kind: Lit: Int(1)
        "#]],
    );
}

#[test]
fn rotl_bitarray_zero_edge_case() {
    let source = r#"
        const bit[5] a = "10001";
        rotl(a, 0);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-34]:
            symbol_id: 8
            ty_span: [15-21]
            init_expr: Expr [26-33]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: Lit: Bitstring("10001")
        ExprStmt [43-54]:
            expr: Expr [43-53]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: BuiltinFunctionCall [43-53]:
                    fn_name_span: [43-47]
                    name: rotl
                    function_ty: def (const bit[5], const int) -> const bit[5]
                    args:
                        Expr [48-49]:
                            ty: const bit[5]
                            const_value: Bitstring("10001")
                            kind: SymbolId(8)
                        Expr [51-52]:
                            ty: const int
                            const_value: Int(0)
                            kind: Lit: Int(0)
    "#]],
    );
}

#[test]
fn rotl_uint() {
    let source = r#"
        const uint[5] a = 17;
        rotl(a, 1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-30]:
                symbol_id: 8
                ty_span: [15-22]
                init_expr: Expr [27-29]:
                    ty: const uint[5]
                    const_value: Int(17)
                    kind: Lit: Int(17)
            ExprStmt [39-50]:
                expr: Expr [39-49]:
                    ty: const bit[5]
                    const_value: Bitstring("00011")
                    kind: BuiltinFunctionCall [39-49]:
                        fn_name_span: [39-43]
                        name: rotl
                        function_ty: def (const bit[5], const int) -> const bit[5]
                        args:
                            Expr [44-45]:
                                ty: const uint[5]
                                const_value: Int(17)
                                kind: SymbolId(8)
                            Expr [47-48]:
                                ty: const int
                                const_value: Int(1)
                                kind: Lit: Int(1)
        "#]],
    );
}

#[test]
fn rotl_uint_negative_distance() {
    let source = r#"
        const uint[5] a = 17;
        rotl(a, -1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-30]:
                symbol_id: 8
                ty_span: [15-22]
                init_expr: Expr [27-29]:
                    ty: const uint[5]
                    const_value: Int(17)
                    kind: Lit: Int(17)
            ExprStmt [39-51]:
                expr: Expr [39-50]:
                    ty: const bit[5]
                    const_value: Bitstring("11000")
                    kind: BuiltinFunctionCall [39-50]:
                        fn_name_span: [39-43]
                        name: rotl
                        function_ty: def (const bit[5], const int) -> const bit[5]
                        args:
                            Expr [44-45]:
                                ty: const uint[5]
                                const_value: Int(17)
                                kind: SymbolId(8)
                            Expr [48-49]:
                                ty: const int
                                const_value: Int(-1)
                                kind: UnaryOpExpr [48-49]:
                                    op: Neg
                                    expr: Expr [48-49]:
                                        ty: const int
                                        kind: Lit: Int(1)
        "#]],
    );
}

#[test]
fn rotl_uint_zero_edge_case() {
    let source = r#"
        const uint[5] a = 17;
        rotl(a, 0);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [15-22]
            init_expr: Expr [27-29]:
                ty: const uint[5]
                const_value: Int(17)
                kind: Lit: Int(17)
        ExprStmt [39-50]:
            expr: Expr [39-49]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: BuiltinFunctionCall [39-49]:
                    fn_name_span: [39-43]
                    name: rotl
                    function_ty: def (const bit[5], const int) -> const bit[5]
                    args:
                        Expr [44-45]:
                            ty: const uint[5]
                            const_value: Int(17)
                            kind: SymbolId(8)
                        Expr [47-48]:
                            ty: const int
                            const_value: Int(0)
                            kind: Lit: Int(0)
    "#]],
    );
}

#[test]
fn rotl_unsized_type_error() {
    let source = r#"
        rotl(17, 2);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-21]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

          x There is no valid overload of `rotl` for inputs: (const int, const int)
          | Overloads available are:
          |     fn (bit[n], int) -> bit[n]
          |     fn (uint[n], int) -> uint[n]
           ,-[test:2:9]
         1 | 
         2 |         rotl(17, 2);
           :         ^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

// ----------------------------------
// rotr

#[test]
fn rotr_bitarray() {
    let source = r#"
        const bit[5] a = "10001";
        rotr(a, 1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-34]:
            symbol_id: 8
            ty_span: [15-21]
            init_expr: Expr [26-33]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: Lit: Bitstring("10001")
        ExprStmt [43-54]:
            expr: Expr [43-53]:
                ty: const bit[5]
                const_value: Bitstring("11000")
                kind: BuiltinFunctionCall [43-53]:
                    fn_name_span: [43-47]
                    name: rotr
                    function_ty: def (const bit[5], const int) -> const bit[5]
                    args:
                        Expr [48-49]:
                            ty: const bit[5]
                            const_value: Bitstring("10001")
                            kind: SymbolId(8)
                        Expr [51-52]:
                            ty: const int
                            const_value: Int(1)
                            kind: Lit: Int(1)
    "#]],
    );
}

#[test]
fn rotr_bitarray_negative_distance() {
    let source = r#"
        const bit[5] a = "10001";
        rotr(a, -1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-34]:
                symbol_id: 8
                ty_span: [15-21]
                init_expr: Expr [26-33]:
                    ty: const bit[5]
                    const_value: Bitstring("10001")
                    kind: Lit: Bitstring("10001")
            ExprStmt [43-55]:
                expr: Expr [43-54]:
                    ty: const bit[5]
                    const_value: Bitstring("00011")
                    kind: BuiltinFunctionCall [43-54]:
                        fn_name_span: [43-47]
                        name: rotr
                        function_ty: def (const bit[5], const int) -> const bit[5]
                        args:
                            Expr [48-49]:
                                ty: const bit[5]
                                const_value: Bitstring("10001")
                                kind: SymbolId(8)
                            Expr [52-53]:
                                ty: const int
                                const_value: Int(-1)
                                kind: UnaryOpExpr [52-53]:
                                    op: Neg
                                    expr: Expr [52-53]:
                                        ty: const int
                                        kind: Lit: Int(1)
        "#]],
    );
}

#[test]
fn rotr_bitarray_zero_edge_case() {
    let source = r#"
        const bit[5] a = "10001";
        rotr(a, 0);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-34]:
            symbol_id: 8
            ty_span: [15-21]
            init_expr: Expr [26-33]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: Lit: Bitstring("10001")
        ExprStmt [43-54]:
            expr: Expr [43-53]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: BuiltinFunctionCall [43-53]:
                    fn_name_span: [43-47]
                    name: rotr
                    function_ty: def (const bit[5], const int) -> const bit[5]
                    args:
                        Expr [48-49]:
                            ty: const bit[5]
                            const_value: Bitstring("10001")
                            kind: SymbolId(8)
                        Expr [51-52]:
                            ty: const int
                            const_value: Int(0)
                            kind: Lit: Int(0)
    "#]],
    );
}

#[test]
fn rotr_uint() {
    let source = r#"
        const uint[5] a = 17;
        rotr(a, 1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [15-22]
            init_expr: Expr [27-29]:
                ty: const uint[5]
                const_value: Int(17)
                kind: Lit: Int(17)
        ExprStmt [39-50]:
            expr: Expr [39-49]:
                ty: const bit[5]
                const_value: Bitstring("11000")
                kind: BuiltinFunctionCall [39-49]:
                    fn_name_span: [39-43]
                    name: rotr
                    function_ty: def (const bit[5], const int) -> const bit[5]
                    args:
                        Expr [44-45]:
                            ty: const uint[5]
                            const_value: Int(17)
                            kind: SymbolId(8)
                        Expr [47-48]:
                            ty: const int
                            const_value: Int(1)
                            kind: Lit: Int(1)
    "#]],
    );
}

#[test]
fn rotr_uint_negative_distance() {
    let source = r#"
        const uint[5] a = 17;
        rotr(a, -1);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-30]:
                symbol_id: 8
                ty_span: [15-22]
                init_expr: Expr [27-29]:
                    ty: const uint[5]
                    const_value: Int(17)
                    kind: Lit: Int(17)
            ExprStmt [39-51]:
                expr: Expr [39-50]:
                    ty: const bit[5]
                    const_value: Bitstring("00011")
                    kind: BuiltinFunctionCall [39-50]:
                        fn_name_span: [39-43]
                        name: rotr
                        function_ty: def (const bit[5], const int) -> const bit[5]
                        args:
                            Expr [44-45]:
                                ty: const uint[5]
                                const_value: Int(17)
                                kind: SymbolId(8)
                            Expr [48-49]:
                                ty: const int
                                const_value: Int(-1)
                                kind: UnaryOpExpr [48-49]:
                                    op: Neg
                                    expr: Expr [48-49]:
                                        ty: const int
                                        kind: Lit: Int(1)
        "#]],
    );
}

#[test]
fn rotr_uint_zero_edge_case() {
    let source = r#"
        const uint[5] a = 17;
        rotr(a, 0);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [15-22]
            init_expr: Expr [27-29]:
                ty: const uint[5]
                const_value: Int(17)
                kind: Lit: Int(17)
        ExprStmt [39-50]:
            expr: Expr [39-49]:
                ty: const bit[5]
                const_value: Bitstring("10001")
                kind: BuiltinFunctionCall [39-49]:
                    fn_name_span: [39-43]
                    name: rotr
                    function_ty: def (const bit[5], const int) -> const bit[5]
                    args:
                        Expr [44-45]:
                            ty: const uint[5]
                            const_value: Int(17)
                            kind: SymbolId(8)
                        Expr [47-48]:
                            ty: const int
                            const_value: Int(0)
                            kind: Lit: Int(0)
    "#]],
    );
}

#[test]
fn rotr_unsized_type_error() {
    let source = r#"
        rotl(17, 2);
    "#;

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-21]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.NoValidOverloadForBuiltinFunction

          x There is no valid overload of `rotl` for inputs: (const int, const int)
          | Overloads available are:
          |     fn (bit[n], int) -> bit[n]
          |     fn (uint[n], int) -> uint[n]
           ,-[test:2:9]
         1 | 
         2 |         rotl(17, 2);
           :         ^^^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

// ----------------------------------
// sin

#[test]
fn sin_float() {
    let source = "
        sin(pi);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-17]:
            expr: Expr [9-16]:
                ty: const float
                const_value: Float(1.2246467991473532e-16)
                kind: BuiltinFunctionCall [9-16]:
                    fn_name_span: [9-12]
                    name: sin
                    function_ty: def (const float) -> const float
                    args:
                        Expr [13-15]:
                            ty: const float
                            const_value: Float(3.141592653589793)
                            kind: SymbolId(2)
    "#]],
    );
}

#[test]
fn sin_angle() {
    let source = "
        const angle a = pi;
        sin(a);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-28]:
            symbol_id: 8
            ty_span: [15-20]
            init_expr: Expr [25-27]:
                ty: const angle
                const_value: Angle(3.141592653589793)
                kind: Cast [0-0]:
                    ty: const angle
                    expr: Expr [25-27]:
                        ty: const float
                        kind: SymbolId(2)
        ExprStmt [37-44]:
            expr: Expr [37-43]:
                ty: const float
                const_value: Float(1.2246467991473532e-16)
                kind: BuiltinFunctionCall [37-43]:
                    fn_name_span: [37-40]
                    name: sin
                    function_ty: def (const angle) -> const float
                    args:
                        Expr [41-42]:
                            ty: const angle
                            const_value: Angle(3.141592653589793)
                            kind: SymbolId(8)
    "#]],
    );
}

// ----------------------------------
// sqrt

#[test]
fn sqrt_float() {
    let source = "
        sqrt(4.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ExprStmt [9-18]:
                expr: Expr [9-17]:
                    ty: const float
                    const_value: Float(2.0)
                    kind: BuiltinFunctionCall [9-17]:
                        fn_name_span: [9-13]
                        name: sqrt
                        function_ty: def (const float) -> const float
                        args:
                            Expr [14-16]:
                                ty: const float
                                const_value: Float(4.0)
                                kind: Lit: Float(4.0)
        "#]],
    );
}

#[test]
fn sqrt_float_domain_error() {
    let source = "
        sqrt(-4.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: Err

        [Qasm.Lowerer.DomainError

          x cannot compute square root of negative floats
           ,-[test:2:9]
         1 | 
         2 |         sqrt(-4.);
           :         ^^^^^^^^^
         3 |     
           `----
        ]"#]],
    );
}

#[test]
fn sqrt_complex() {
    let source = "
        sqrt(7 + 24 im);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
            ExprStmt [9-25]:
                expr: Expr [9-24]:
                    ty: const complex[float]
                    const_value: Complex(4.0, 3.0)
                    kind: BuiltinFunctionCall [9-24]:
                        fn_name_span: [9-13]
                        name: sqrt
                        function_ty: def (const complex[float]) -> const complex[float]
                        args:
                            Expr [14-23]:
                                ty: const complex[float]
                                const_value: Complex(7.0, 24.0)
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [14-15]:
                                        ty: const complex[float]
                                        kind: Lit: Complex(7.0, 0.0)
                                    rhs: Expr [18-23]:
                                        ty: const complex[float]
                                        kind: Lit: Complex(0.0, 24.0)
        "#]],
    );
}

// ----------------------------------
// tan

#[test]
fn tan_float() {
    let source = "
        tan(pi / 4.);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ExprStmt [9-22]:
            expr: Expr [9-21]:
                ty: const float
                const_value: Float(0.9999999999999999)
                kind: BuiltinFunctionCall [9-21]:
                    fn_name_span: [9-12]
                    name: tan
                    function_ty: def (const float) -> const float
                    args:
                        Expr [13-20]:
                            ty: const float
                            const_value: Float(0.7853981633974483)
                            kind: BinaryOpExpr:
                                op: Div
                                lhs: Expr [13-15]:
                                    ty: const float
                                    kind: SymbolId(2)
                                rhs: Expr [18-20]:
                                    ty: const float
                                    kind: Lit: Float(4.0)
    "#]],
    );
}

#[test]
fn tan_angle() {
    let source = "
        const angle a = pi / 4.;
        tan(a);
    ";

    check_stmt_kinds(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-33]:
            symbol_id: 8
            ty_span: [15-20]
            init_expr: Expr [25-32]:
                ty: const angle
                const_value: Angle(0.7853981633974483)
                kind: Cast [0-0]:
                    ty: const angle
                    expr: Expr [25-32]:
                        ty: const float
                        kind: BinaryOpExpr:
                            op: Div
                            lhs: Expr [25-27]:
                                ty: const float
                                kind: SymbolId(2)
                            rhs: Expr [30-32]:
                                ty: const float
                                kind: Lit: Float(4.0)
        ExprStmt [42-49]:
            expr: Expr [42-48]:
                ty: const float
                const_value: Float(0.9999999999999999)
                kind: BuiltinFunctionCall [42-48]:
                    fn_name_span: [42-45]
                    name: tan
                    function_ty: def (const angle) -> const float
                    args:
                        Expr [46-47]:
                            ty: const angle
                            const_value: Angle(0.7853981633974483)
                            kind: SymbolId(8)
    "#]],
    );
}
