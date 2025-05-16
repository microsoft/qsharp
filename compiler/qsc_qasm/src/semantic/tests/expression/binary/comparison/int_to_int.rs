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
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: int
                    kind: Lit: Int(5)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Lit: Int(3)
            [9] Symbol [32-33]:
                name: y
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-62]:
                symbol_id: 10
                ty_span: [47-51]
                init_expr: Expr [56-61]:
                    ty: bool
                    kind: BinaryOpExpr:
                        op: Gt
                        lhs: Expr [56-57]:
                            ty: int
                            kind: SymbolId(8)
                        rhs: Expr [60-61]:
                            ty: int
                            kind: SymbolId(9)
            [10] Symbol [52-53]:
                name: f
                type: bool
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
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: int
                    kind: Lit: Int(5)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Lit: Int(3)
            [9] Symbol [32-33]:
                name: y
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 10
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: bool
                    kind: BinaryOpExpr:
                        op: Gte
                        lhs: Expr [56-57]:
                            ty: int
                            kind: SymbolId(8)
                        rhs: Expr [61-62]:
                            ty: int
                            kind: SymbolId(9)
            [10] Symbol [52-53]:
                name: e
                type: bool
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
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: int
                    kind: Lit: Int(5)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Lit: Int(3)
            [9] Symbol [32-33]:
                name: y
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-62]:
                symbol_id: 10
                ty_span: [47-51]
                init_expr: Expr [56-61]:
                    ty: bool
                    kind: BinaryOpExpr:
                        op: Lt
                        lhs: Expr [56-57]:
                            ty: int
                            kind: SymbolId(8)
                        rhs: Expr [60-61]:
                            ty: int
                            kind: SymbolId(9)
            [10] Symbol [52-53]:
                name: a
                type: bool
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
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: int
                    kind: Lit: Int(5)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Lit: Int(3)
            [9] Symbol [32-33]:
                name: y
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 10
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: bool
                    kind: BinaryOpExpr:
                        op: Lte
                        lhs: Expr [56-57]:
                            ty: int
                            kind: SymbolId(8)
                        rhs: Expr [61-62]:
                            ty: int
                            kind: SymbolId(9)
            [10] Symbol [52-53]:
                name: c
                type: bool
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
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: int
                    kind: Lit: Int(5)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Lit: Int(3)
            [9] Symbol [32-33]:
                name: y
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 10
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: bool
                    kind: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [56-57]:
                            ty: int
                            kind: SymbolId(8)
                        rhs: Expr [61-62]:
                            ty: int
                            kind: SymbolId(9)
            [10] Symbol [52-53]:
                name: b
                type: bool
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
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: int
                    kind: Lit: Int(5)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Lit: Int(3)
            [9] Symbol [32-33]:
                name: y
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 10
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: bool
                    kind: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [56-57]:
                            ty: int
                            kind: SymbolId(8)
                        rhs: Expr [61-62]:
                            ty: int
                            kind: SymbolId(9)
            [10] Symbol [52-53]:
                name: d
                type: bool
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}
