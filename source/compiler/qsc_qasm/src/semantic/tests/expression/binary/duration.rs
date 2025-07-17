// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::{check_stmt_kind, check_stmt_kinds};
use expect_test::expect;

#[test]
fn addition_with_units_normalizes_correctly() {
    let input = "
        duration x = 1 s + 3 ms + 6 us + 9 ns;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-47]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-46]:
                    ty: duration
                    const_value: Duration(1003006009.0 ns)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [22-39]:
                            ty: duration
                            const_value: Duration(1003006.0 us)
                            kind: BinaryOpExpr:
                                op: Add
                                lhs: Expr [22-32]:
                                    ty: duration
                                    const_value: Duration(1003.0 ms)
                                    kind: BinaryOpExpr:
                                        op: Add
                                        lhs: Expr [22-25]:
                                            ty: duration
                                            kind: Lit: Duration(1.0 s)
                                        rhs: Expr [28-32]:
                                            ty: duration
                                            kind: Lit: Duration(3.0 ms)
                                rhs: Expr [35-39]:
                                    ty: duration
                                    kind: Lit: Duration(6.0 us)
                        rhs: Expr [42-46]:
                            ty: duration
                            kind: Lit: Duration(9.0 ns)
        "#]],
    );
}

#[test]
fn addition_of_two_durations_returns_duration() {
    let input = "
        duration x = 2 s + 3 s;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-32]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-31]:
                    ty: duration
                    const_value: Duration(5.0 s)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [22-25]:
                            ty: duration
                            kind: Lit: Duration(2.0 s)
                        rhs: Expr [28-31]:
                            ty: duration
                            kind: Lit: Duration(3.0 s)
        "#]],
    );
}

#[test]
fn addition_with_duration_and_non_duration_value_errors() {
    let input = "
        duration a;
        int b;
        duration x = a + b;
    ";

    check_stmt_kind(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-20]:
                            symbol_id: 8
                            ty_span: [9-17]
                            init_expr: Expr [9-20]:
                                ty: duration
                                const_value: Duration(0.0 s)
                                kind: Lit: Duration(0.0 s)
                    Stmt [29-35]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [29-35]:
                            symbol_id: 9
                            ty_span: [29-32]
                            init_expr: Expr [29-35]:
                                ty: const int
                                kind: Lit: Int(0)
                    Stmt [44-63]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [44-63]:
                            symbol_id: 10
                            ty_span: [44-52]
                            init_expr: Expr [57-62]:
                                ty: unknown
                                kind: Err

            [Qasm.Lowerer.CannotApplyOperatorToTypes

              x cannot apply operator Add to types duration and int
               ,-[test:4:22]
             3 |         int b;
             4 |         duration x = a + b;
               :                      ^^^^^
             5 |     
               `----
            ]"#]],
    );
}

#[test]
fn addition_assign_op_errors_as_duration_is_const() {
    let input = "
        duration a;
        duration b;
        a += b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-20]:
                            symbol_id: 8
                            ty_span: [9-17]
                            init_expr: Expr [9-20]:
                                ty: duration
                                const_value: Duration(0.0 s)
                                kind: Lit: Duration(0.0 s)
                    Stmt [29-40]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [29-40]:
                            symbol_id: 9
                            ty_span: [29-37]
                            init_expr: Expr [29-40]:
                                ty: duration
                                const_value: Duration(0.0 s)
                                kind: Lit: Duration(0.0 s)
                    Stmt [49-56]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable a
               ,-[test:4:9]
             3 |         duration b;
             4 |         a += b;
               :         ^
             5 |     
               `----
              help: mutable variables must be declared without the keyword `const`
            ]"#]],
    );
}

#[test]
fn subtraction() {
    let input = "
        duration x = 3 s - 2 s;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-32]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-31]:
                    ty: duration
                    const_value: Duration(1.0 s)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [22-25]:
                            ty: duration
                            kind: Lit: Duration(3.0 s)
                        rhs: Expr [28-31]:
                            ty: duration
                            kind: Lit: Duration(2.0 s)
        "#]],
    );
}

#[test]
fn subtraction_can_result_in_negative_duration() {
    let input = "
        duration x = 2 s - 3 s;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-32]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-31]:
                    ty: duration
                    const_value: Duration(-1.0 s)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [22-25]:
                            ty: duration
                            kind: Lit: Duration(2.0 s)
                        rhs: Expr [28-31]:
                            ty: duration
                            kind: Lit: Duration(3.0 s)
        "#]],
    );
}

#[test]
fn subtraction_assign_op_errors_as_duration_is_const() {
    let input = "
        duration a;
        duration b;
        a -= b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-20]:
                            symbol_id: 8
                            ty_span: [9-17]
                            init_expr: Expr [9-20]:
                                ty: duration
                                const_value: Duration(0.0 s)
                                kind: Lit: Duration(0.0 s)
                    Stmt [29-40]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [29-40]:
                            symbol_id: 9
                            ty_span: [29-37]
                            init_expr: Expr [29-40]:
                                ty: duration
                                const_value: Duration(0.0 s)
                                kind: Lit: Duration(0.0 s)
                    Stmt [49-56]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable a
               ,-[test:4:9]
             3 |         duration b;
             4 |         a -= b;
               :         ^
             5 |     
               `----
              help: mutable variables must be declared without the keyword `const`
            ]"#]],
    );
}

// todo, shouldn't get a duplicate
#[test]
fn multiplication_by_duration_is_not_supported() {
    let input = "
        duration a = 1ms;
        duration b = 2ms;
        duration x = a * b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-26]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-26]:
                            symbol_id: 8
                            ty_span: [9-17]
                            init_expr: Expr [22-25]:
                                ty: duration
                                const_value: Duration(1.0 ms)
                                kind: Lit: Duration(1.0 ms)
                    Stmt [35-52]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [35-52]:
                            symbol_id: 9
                            ty_span: [35-43]
                            init_expr: Expr [48-51]:
                                ty: duration
                                const_value: Duration(2.0 ms)
                                kind: Lit: Duration(2.0 ms)
                    Stmt [61-80]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [61-80]:
                            symbol_id: 10
                            ty_span: [61-69]
                            init_expr: Expr [74-79]:
                                ty: unknown
                                kind: Err

            [Qasm.Lowerer.CannotApplyOperatorToTypes

              x cannot apply operator Mul to types duration and duration
               ,-[test:4:22]
             3 |         duration b = 2ms;
             4 |         duration x = a * b;
               :                      ^^^^^
             5 |     
               `----
            ]"#]],
    );
}

#[test]
fn multiplication_duration_by_int() {
    let input = "
        duration a = 2ms;
        const int b = 3;
        duration x = a * b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-26]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-25]:
                    ty: duration
                    const_value: Duration(2.0 ms)
                    kind: Lit: Duration(2.0 ms)
            ClassicalDeclarationStmt [35-51]:
                symbol_id: 9
                ty_span: [41-44]
                init_expr: Expr [49-50]:
                    ty: const int
                    const_value: Int(3)
                    kind: Lit: Int(3)
            ClassicalDeclarationStmt [60-79]:
                symbol_id: 10
                ty_span: [60-68]
                init_expr: Expr [73-78]:
                    ty: duration
                    const_value: Duration(6.0 ms)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [73-74]:
                            ty: duration
                            kind: SymbolId(8)
                        rhs: Expr [77-78]:
                            ty: const int
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multiplication_int_by_duration() {
    let input = "
        duration a = 2ms;
        const int b = 3;
        duration x = b * a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-26]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-25]:
                    ty: duration
                    const_value: Duration(2.0 ms)
                    kind: Lit: Duration(2.0 ms)
            ClassicalDeclarationStmt [35-51]:
                symbol_id: 9
                ty_span: [41-44]
                init_expr: Expr [49-50]:
                    ty: const int
                    const_value: Int(3)
                    kind: Lit: Int(3)
            ClassicalDeclarationStmt [60-79]:
                symbol_id: 10
                ty_span: [60-68]
                init_expr: Expr [73-78]:
                    ty: duration
                    const_value: Duration(6.0 ms)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [73-74]:
                            ty: const int
                            kind: SymbolId(9)
                        rhs: Expr [77-78]:
                            ty: duration
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn multiplication_duration_by_float() {
    let input = "
        duration a = 2ms;
        const float b = 3.0;
        duration x = a * b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-26]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-25]:
                    ty: duration
                    const_value: Duration(2.0 ms)
                    kind: Lit: Duration(2.0 ms)
            ClassicalDeclarationStmt [35-55]:
                symbol_id: 9
                ty_span: [41-46]
                init_expr: Expr [51-54]:
                    ty: const float
                    const_value: Float(3.0)
                    kind: Lit: Float(3.0)
            ClassicalDeclarationStmt [64-83]:
                symbol_id: 10
                ty_span: [64-72]
                init_expr: Expr [77-82]:
                    ty: duration
                    const_value: Duration(6.0 ms)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [77-78]:
                            ty: duration
                            kind: SymbolId(8)
                        rhs: Expr [81-82]:
                            ty: const float
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multiplication_float_by_duration() {
    let input = "
        duration a = 2ms;
        const float b = 3.0;
        duration x = b * a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-26]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-25]:
                    ty: duration
                    const_value: Duration(2.0 ms)
                    kind: Lit: Duration(2.0 ms)
            ClassicalDeclarationStmt [35-55]:
                symbol_id: 9
                ty_span: [41-46]
                init_expr: Expr [51-54]:
                    ty: const float
                    const_value: Float(3.0)
                    kind: Lit: Float(3.0)
            ClassicalDeclarationStmt [64-83]:
                symbol_id: 10
                ty_span: [64-72]
                init_expr: Expr [77-82]:
                    ty: duration
                    const_value: Duration(6.0 ms)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [77-78]:
                            ty: const float
                            kind: SymbolId(9)
                        rhs: Expr [81-82]:
                            ty: duration
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn multiplication_assign_op_errors_as_duration_is_const() {
    let input = "
        duration a = 2 ns;
        duration b = 3 ns;
        a *= b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-27]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-27]:
                            symbol_id: 8
                            ty_span: [9-17]
                            init_expr: Expr [22-26]:
                                ty: duration
                                const_value: Duration(2.0 ns)
                                kind: Lit: Duration(2.0 ns)
                    Stmt [36-54]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [36-54]:
                            symbol_id: 9
                            ty_span: [36-44]
                            init_expr: Expr [49-53]:
                                ty: duration
                                const_value: Duration(3.0 ns)
                                kind: Lit: Duration(3.0 ns)
                    Stmt [63-70]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable a
               ,-[test:4:9]
             3 |         duration b = 3 ns;
             4 |         a *= b;
               :         ^
             5 |     
               `----
              help: mutable variables must be declared without the keyword `const`
            ]"#]],
    );
}

#[test]
fn division_duration_by_duration_is_float() {
    let input = "
        duration a = 12 ns;
        duration b = 4 ns;
        float x = a / b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-28]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-27]:
                    ty: duration
                    const_value: Duration(12.0 ns)
                    kind: Lit: Duration(12.0 ns)
            ClassicalDeclarationStmt [37-55]:
                symbol_id: 9
                ty_span: [37-45]
                init_expr: Expr [50-54]:
                    ty: duration
                    const_value: Duration(4.0 ns)
                    kind: Lit: Duration(4.0 ns)
            ClassicalDeclarationStmt [64-80]:
                symbol_id: 10
                ty_span: [64-69]
                init_expr: Expr [74-79]:
                    ty: float
                    const_value: Float(3.0)
                    kind: BinaryOpExpr:
                        op: Div
                        lhs: Expr [74-75]:
                            ty: duration
                            kind: SymbolId(8)
                        rhs: Expr [78-79]:
                            ty: duration
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn division_duration_by_int_is_duration() {
    let input = "
        duration a = 12 ns;
        const int b = 4;
        duration x = a / b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-28]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-27]:
                    ty: duration
                    const_value: Duration(12.0 ns)
                    kind: Lit: Duration(12.0 ns)
            ClassicalDeclarationStmt [37-53]:
                symbol_id: 9
                ty_span: [43-46]
                init_expr: Expr [51-52]:
                    ty: const int
                    const_value: Int(4)
                    kind: Lit: Int(4)
            ClassicalDeclarationStmt [62-81]:
                symbol_id: 10
                ty_span: [62-70]
                init_expr: Expr [75-80]:
                    ty: duration
                    const_value: Duration(3.0 ns)
                    kind: BinaryOpExpr:
                        op: Div
                        lhs: Expr [75-76]:
                            ty: duration
                            kind: SymbolId(8)
                        rhs: Expr [79-80]:
                            ty: const int
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn division_duration_by_float_is_duration() {
    let input = "
        duration a = 12 ns;
        const float b = 4.0;
        duration x = a / b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-28]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [22-27]:
                    ty: duration
                    const_value: Duration(12.0 ns)
                    kind: Lit: Duration(12.0 ns)
            ClassicalDeclarationStmt [37-57]:
                symbol_id: 9
                ty_span: [43-48]
                init_expr: Expr [53-56]:
                    ty: const float
                    const_value: Float(4.0)
                    kind: Lit: Float(4.0)
            ClassicalDeclarationStmt [66-85]:
                symbol_id: 10
                ty_span: [66-74]
                init_expr: Expr [79-84]:
                    ty: duration
                    const_value: Duration(3.0 ns)
                    kind: BinaryOpExpr:
                        op: Div
                        lhs: Expr [79-80]:
                            ty: duration
                            kind: SymbolId(8)
                        rhs: Expr [83-84]:
                            ty: const float
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn division_assign_op_errors_as_duration_is_const() {
    let input = "
        duration a = 12 ns;
        const float b = 3.0;
        a /= b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-28]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-28]:
                            symbol_id: 8
                            ty_span: [9-17]
                            init_expr: Expr [22-27]:
                                ty: duration
                                const_value: Duration(12.0 ns)
                                kind: Lit: Duration(12.0 ns)
                    Stmt [37-57]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [37-57]:
                            symbol_id: 9
                            ty_span: [43-48]
                            init_expr: Expr [53-56]:
                                ty: const float
                                const_value: Float(3.0)
                                kind: Lit: Float(3.0)
                    Stmt [66-73]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable a
               ,-[test:4:9]
             3 |         const float b = 3.0;
             4 |         a /= b;
               :         ^
             5 |     
               `----
              help: mutable variables must be declared without the keyword `const`
            ]"#]],
    );
}
