// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn logical_and() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = x && y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-72]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-71]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [65-66]:
                            ty: Bool(false)
                            kind: SymbolId(6)
                        rhs: Expr [70-71]:
                            ty: Bool(false)
                            kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn logical_or() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = x || y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-72]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-71]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [65-66]:
                            ty: Bool(false)
                            kind: SymbolId(6)
                        rhs: Expr [70-71]:
                            ty: Bool(false)
                            kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn unop_not_logical_and_unop_not() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = !x && !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-74]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-73]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [66-67]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [66-67]:
                                    ty: Bool(false)
                                    kind: SymbolId(6)
                        rhs: Expr [72-73]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [72-73]:
                                    ty: Bool(false)
                                    kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn unop_not_logical_or_unop_not() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = !x || !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-74]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-73]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [66-67]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [66-67]:
                                    ty: Bool(false)
                                    kind: SymbolId(6)
                        rhs: Expr [72-73]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [72-73]:
                                    ty: Bool(false)
                                    kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn unop_not_logical_and() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = !x && y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-73]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-72]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [66-67]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [66-67]:
                                    ty: Bool(false)
                                    kind: SymbolId(6)
                        rhs: Expr [71-72]:
                            ty: Bool(false)
                            kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn unop_not_logical_or() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = !x || y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-73]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-72]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [66-67]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [66-67]:
                                    ty: Bool(false)
                                    kind: SymbolId(6)
                        rhs: Expr [71-72]:
                            ty: Bool(false)
                            kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn logical_and_unop_not() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = x && !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-73]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-72]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [65-66]:
                            ty: Bool(false)
                            kind: SymbolId(6)
                        rhs: Expr [71-72]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [71-72]:
                                    ty: Bool(false)
                                    kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

#[test]
fn logical_or_unop_not() {
    let input = "
        bool x = true;
        bool y = false;
        bool a = x || !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-46]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            [7] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [56-73]:
                symbol_id: 8
                ty_span: [56-60]
                init_expr: Expr [65-72]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [65-66]:
                            ty: Bool(false)
                            kind: SymbolId(6)
                        rhs: Expr [71-72]:
                            ty: Bool(false)
                            kind: UnaryOpExpr:
                                op: NotL
                                expr: Expr [71-72]:
                                    ty: Bool(false)
                                    kind: SymbolId(7)
            [8] Symbol [61-62]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}
