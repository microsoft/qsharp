// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn greater_than() {
    let input = "
        int x = 5;
        int y = 3;
        bool f = x > y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            [6] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            [7] Symbol [32-33]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-62]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-61]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Gt
                        lhs: Expr [56-57]:
                            ty: Int(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [60-61]:
                            ty: Int(None, false)
                            kind: SymbolId(7)
            [8] Symbol [52-53]:
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
        int x = 5;
        int y = 3;
        bool e = x >= y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            [6] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            [7] Symbol [32-33]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Gte
                        lhs: Expr [56-57]:
                            ty: Int(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [61-62]:
                            ty: Int(None, false)
                            kind: SymbolId(7)
            [8] Symbol [52-53]:
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
        int x = 5;
        int y = 3;
        bool a = x < y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            [6] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            [7] Symbol [32-33]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-62]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-61]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Lt
                        lhs: Expr [56-57]:
                            ty: Int(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [60-61]:
                            ty: Int(None, false)
                            kind: SymbolId(7)
            [8] Symbol [52-53]:
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
        int x = 5;
        int y = 3;
        bool c = x <= y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            [6] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            [7] Symbol [32-33]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Lte
                        lhs: Expr [56-57]:
                            ty: Int(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [61-62]:
                            ty: Int(None, false)
                            kind: SymbolId(7)
            [8] Symbol [52-53]:
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
        int x = 5;
        int y = 3;
        bool b = x == y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            [6] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            [7] Symbol [32-33]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [56-57]:
                            ty: Int(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [61-62]:
                            ty: Int(None, false)
                            kind: SymbolId(7)
            [8] Symbol [52-53]:
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
        int x = 5;
        int y = 3;
        bool d = x != y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            [6] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            [7] Symbol [32-33]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [56-57]:
                            ty: Int(None, false)
                            kind: SymbolId(6)
                        rhs: Expr [61-62]:
                            ty: Int(None, false)
                            kind: SymbolId(7)
            [8] Symbol [52-53]:
                name: d
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}
