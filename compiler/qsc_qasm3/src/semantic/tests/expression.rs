// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod binary;
mod implicit_cast_from_angle;
mod implicit_cast_from_bit;
mod implicit_cast_from_bitarray;
mod implicit_cast_from_bool;
mod implicit_cast_from_float;
mod implicit_cast_from_int;

use expect_test::expect;

use super::check_stmt_kinds;

#[test]
fn a() {
    check_stmt_kinds(
        r#"
        true && false;
        false || true;
        !true;
        "#,
        &expect![[r#"
            ExprStmt [9-23]:
                expr: Expr [9-22]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: AndL
                        lhs: Expr [9-13]:
                            ty: Bool(true)
                            kind: Lit: Bool(true)
                        rhs: Expr [17-22]:
                            ty: Bool(true)
                            kind: Lit: Bool(false)
            ExprStmt [32-46]:
                expr: Expr [32-45]:
                    ty: Bool(false)
                    kind: BinaryOpExpr:
                        op: OrL
                        lhs: Expr [32-37]:
                            ty: Bool(true)
                            kind: Lit: Bool(false)
                        rhs: Expr [41-45]:
                            ty: Bool(true)
                            kind: Lit: Bool(true)
            ExprStmt [55-61]:
                expr: Expr [56-60]:
                    ty: Bool(true)
                    kind: UnaryOpExpr:
                        op: NotL
                        expr: Expr [56-60]:
                            ty: Bool(true)
                            kind: Lit: Bool(true)
        "#]],
    );
}
