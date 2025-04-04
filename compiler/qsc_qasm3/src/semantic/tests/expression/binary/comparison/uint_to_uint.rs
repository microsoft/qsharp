// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn greater_than() {
    let input = "
        uint x = 5;
        uint y = 3;
        bool f = x > y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [18-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(5)
            [8] Symbol [14-15]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, true)
                    kind: Lit: Int(3)
            [9] Symbol [34-35]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [49-64]:
                symbol_id: 10
                ty_span: [49-53]
                init_expr: Expr [58-63]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Gt
                        lhs: Expr [58-59]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [62-63]:
                            ty: UInt(None, false)
                            kind: SymbolId(9)
            [10] Symbol [54-55]:
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
        uint x = 5;
        uint y = 3;
        bool e = x >= y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [18-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(5)
            [8] Symbol [14-15]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, true)
                    kind: Lit: Int(3)
            [9] Symbol [34-35]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [49-65]:
                symbol_id: 10
                ty_span: [49-53]
                init_expr: Expr [58-64]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Gte
                        lhs: Expr [58-59]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [63-64]:
                            ty: UInt(None, false)
                            kind: SymbolId(9)
            [10] Symbol [54-55]:
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
        uint x = 5;
        uint y = 3;
        bool a = x < y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [18-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(5)
            [8] Symbol [14-15]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, true)
                    kind: Lit: Int(3)
            [9] Symbol [34-35]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [49-64]:
                symbol_id: 10
                ty_span: [49-53]
                init_expr: Expr [58-63]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Lt
                        lhs: Expr [58-59]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [62-63]:
                            ty: UInt(None, false)
                            kind: SymbolId(9)
            [10] Symbol [54-55]:
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
        uint x = 5;
        uint y = 3;
        bool c = x <= y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [18-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(5)
            [8] Symbol [14-15]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, true)
                    kind: Lit: Int(3)
            [9] Symbol [34-35]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [49-65]:
                symbol_id: 10
                ty_span: [49-53]
                init_expr: Expr [58-64]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Lte
                        lhs: Expr [58-59]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [63-64]:
                            ty: UInt(None, false)
                            kind: SymbolId(9)
            [10] Symbol [54-55]:
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
        uint x = 5;
        uint y = 3;
        bool b = x == y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [18-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(5)
            [8] Symbol [14-15]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, true)
                    kind: Lit: Int(3)
            [9] Symbol [34-35]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [49-65]:
                symbol_id: 10
                ty_span: [49-53]
                init_expr: Expr [58-64]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [58-59]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [63-64]:
                            ty: UInt(None, false)
                            kind: SymbolId(9)
            [10] Symbol [54-55]:
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
        uint x = 5;
        uint y = 3;
        bool d = x != y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [18-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(5)
            [8] Symbol [14-15]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, true)
                    kind: Lit: Int(3)
            [9] Symbol [34-35]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [49-65]:
                symbol_id: 10
                ty_span: [49-53]
                init_expr: Expr [58-64]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [58-59]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [63-64]:
                            ty: UInt(None, false)
                            kind: SymbolId(9)
            [10] Symbol [54-55]:
                name: d
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}
