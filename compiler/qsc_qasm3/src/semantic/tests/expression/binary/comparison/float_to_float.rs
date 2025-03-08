// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn greater_than() {
    let input = "
        float x = 5.;
        float y = 3.;
        bool f = x > y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-22]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-21]:
                    ty: Float(None, true)
                    kind: Lit: Float(5.0)
            [6] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [31-44]:
                symbol_id: 7
                ty_span: [31-36]
                init_expr: Expr [41-43]:
                    ty: Float(None, true)
                    kind: Lit: Float(3.0)
            [7] Symbol [37-38]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [53-68]:
                symbol_id: 8
                ty_span: [53-57]
                init_expr: Expr [62-67]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Gt
                        lhs: Expr [62-63]:
                            ty: Float(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [66-67]:
                            ty: Float(None, false)
                            kind: SymbolId(7)
            [8] Symbol [58-59]:
                name: f
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn greater_than_equals() {
    let input = "
        float x = 5.;
        float y = 3.;
        bool e = x >= y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-22]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-21]:
                    ty: Float(None, true)
                    kind: Lit: Float(5.0)
            [6] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [31-44]:
                symbol_id: 7
                ty_span: [31-36]
                init_expr: Expr [41-43]:
                    ty: Float(None, true)
                    kind: Lit: Float(3.0)
            [7] Symbol [37-38]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [53-69]:
                symbol_id: 8
                ty_span: [53-57]
                init_expr: Expr [62-68]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Gte
                        lhs: Expr [62-63]:
                            ty: Float(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [67-68]:
                            ty: Float(None, false)
                            kind: SymbolId(7)
            [8] Symbol [58-59]:
                name: e
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn less_than() {
    let input = "
        float x = 5.;
        float y = 3.;
        bool a = x < y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-22]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-21]:
                    ty: Float(None, true)
                    kind: Lit: Float(5.0)
            [6] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [31-44]:
                symbol_id: 7
                ty_span: [31-36]
                init_expr: Expr [41-43]:
                    ty: Float(None, true)
                    kind: Lit: Float(3.0)
            [7] Symbol [37-38]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [53-68]:
                symbol_id: 8
                ty_span: [53-57]
                init_expr: Expr [62-67]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Lt
                        lhs: Expr [62-63]:
                            ty: Float(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [66-67]:
                            ty: Float(None, false)
                            kind: SymbolId(7)
            [8] Symbol [58-59]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn less_than_equals() {
    let input = "
        float x = 5.;
        float y = 3.;
        bool c = x <= y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-22]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-21]:
                    ty: Float(None, true)
                    kind: Lit: Float(5.0)
            [6] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [31-44]:
                symbol_id: 7
                ty_span: [31-36]
                init_expr: Expr [41-43]:
                    ty: Float(None, true)
                    kind: Lit: Float(3.0)
            [7] Symbol [37-38]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [53-69]:
                symbol_id: 8
                ty_span: [53-57]
                init_expr: Expr [62-68]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Lte
                        lhs: Expr [62-63]:
                            ty: Float(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [67-68]:
                            ty: Float(None, false)
                            kind: SymbolId(7)
            [8] Symbol [58-59]:
                name: c
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn equals() {
    let input = "
        float x = 5.;
        float y = 3.;
        bool b = x == y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-22]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-21]:
                    ty: Float(None, true)
                    kind: Lit: Float(5.0)
            [6] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [31-44]:
                symbol_id: 7
                ty_span: [31-36]
                init_expr: Expr [41-43]:
                    ty: Float(None, true)
                    kind: Lit: Float(3.0)
            [7] Symbol [37-38]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [53-69]:
                symbol_id: 8
                ty_span: [53-57]
                init_expr: Expr [62-68]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [62-63]:
                            ty: Float(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [67-68]:
                            ty: Float(None, false)
                            kind: SymbolId(7)
            [8] Symbol [58-59]:
                name: b
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn not_equals() {
    let input = "
        float x = 5.;
        float y = 3.;
        bool d = x != y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-22]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-21]:
                    ty: Float(None, true)
                    kind: Lit: Float(5.0)
            [6] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [31-44]:
                symbol_id: 7
                ty_span: [31-36]
                init_expr: Expr [41-43]:
                    ty: Float(None, true)
                    kind: Lit: Float(3.0)
            [7] Symbol [37-38]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [53-69]:
                symbol_id: 8
                ty_span: [53-57]
                init_expr: Expr [62-68]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [62-63]:
                            ty: Float(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [67-68]:
                            ty: Float(None, false)
                            kind: SymbolId(7)
            [8] Symbol [58-59]:
                name: d
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}
