// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn logical_and() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = x && y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [56-57]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [56-57]:
                                    ty: Bit(false)
                                    kind: SymbolId(6)
                        rhs: Expr [61-62]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [61-62]:
                                    ty: Bit(false)
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
fn logical_or() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = x || y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-63]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-62]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [56-57]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [56-57]:
                                    ty: Bit(false)
                                    kind: SymbolId(6)
                        rhs: Expr [61-62]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [61-62]:
                                    ty: Bit(false)
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
fn unop_not_logical_and_unop_not() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = !x && !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-65]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-64]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [57-58]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [57-58]:
                                op: NotL
                                expr: Expr [57-58]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [57-58]:
                                            ty: Bit(false)
                                            kind: SymbolId(6)
                        rhs: Expr [63-64]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [63-64]:
                                op: NotL
                                expr: Expr [63-64]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [63-64]:
                                            ty: Bit(false)
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
fn unop_not_logical_or_unop_not() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = !x || !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-65]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-64]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [57-58]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [57-58]:
                                op: NotL
                                expr: Expr [57-58]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [57-58]:
                                            ty: Bit(false)
                                            kind: SymbolId(6)
                        rhs: Expr [63-64]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [63-64]:
                                op: NotL
                                expr: Expr [63-64]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [63-64]:
                                            ty: Bit(false)
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
fn unop_not_logical_and() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = !x && y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-64]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-63]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [57-58]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [57-58]:
                                op: NotL
                                expr: Expr [57-58]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [57-58]:
                                            ty: Bit(false)
                                            kind: SymbolId(6)
                        rhs: Expr [62-63]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [62-63]:
                                    ty: Bit(false)
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
fn unop_not_logical_or() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = !x || y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-64]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-63]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [57-58]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [57-58]:
                                op: NotL
                                expr: Expr [57-58]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [57-58]:
                                            ty: Bit(false)
                                            kind: SymbolId(6)
                        rhs: Expr [62-63]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [62-63]:
                                    ty: Bit(false)
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
fn logical_and_unop_not() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = x && !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-64]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-63]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [56-57]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [56-57]:
                                    ty: Bit(false)
                                    kind: SymbolId(6)
                        rhs: Expr [62-63]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [62-63]:
                                op: NotL
                                expr: Expr [62-63]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [62-63]:
                                            ty: Bit(false)
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
fn logical_or_unop_not() {
    let input = "
        bit x = 1;
        bit y = 0;
        bool a = x || !y;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 6
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 7
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Bit(true)
                    kind: Lit: Int(0)
            [7] Symbol [32-33]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [47-64]:
                symbol_id: 8
                ty_span: [47-51]
                init_expr: Expr [56-63]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [56-57]:
                            ty: Bool(false)
                            kind: Cast [0-0]:
                                ty: Bool(false)
                                expr: Expr [56-57]:
                                    ty: Bit(false)
                                    kind: SymbolId(6)
                        rhs: Expr [62-63]:
                            ty: Bool(false)
                            kind: UnaryOpExpr [62-63]:
                                op: NotL
                                expr: Expr [62-63]:
                                    ty: Bool(false)
                                    kind: Cast [0-0]:
                                        ty: Bool(false)
                                        expr: Expr [62-63]:
                                            ty: Bit(false)
                                            kind: SymbolId(7)
            [8] Symbol [52-53]:
                name: a
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}
