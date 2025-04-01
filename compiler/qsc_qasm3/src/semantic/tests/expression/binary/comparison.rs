// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod bit_to_bit;
mod bool_to_bool;
mod float_to_float;
mod int_to_int;

mod uint_to_uint;

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

#[test]
fn bitarray_var_comparisons_can_be_translated() {
    let input = r#"
        bit[1] x = "1";
        bit[1] y = "0";
        bool f = x > y;
        bool e = x >= y;
        bool a = x < y;
        bool c = x <= y;
        bool b = x == y;
        bool d = x != y;
    "#;

    check_stmt_kinds(input, &expect![[r#"
        ClassicalDeclarationStmt [9-24]:
            symbol_id: 8
            ty_span: [9-15]
            init_expr: Expr [20-23]:
                ty: BitArray(One(1), false)
                kind: Lit: Bitstring("1")
        ClassicalDeclarationStmt [33-48]:
            symbol_id: 9
            ty_span: [33-39]
            init_expr: Expr [44-47]:
                ty: BitArray(One(1), false)
                kind: Lit: Bitstring("0")
        ClassicalDeclarationStmt [57-72]:
            symbol_id: 10
            ty_span: [57-61]
            init_expr: Expr [66-71]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Gt
                    lhs: Expr [66-67]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [66-67]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [70-71]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [70-71]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(9)
        ClassicalDeclarationStmt [81-97]:
            symbol_id: 11
            ty_span: [81-85]
            init_expr: Expr [90-96]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Gte
                    lhs: Expr [90-91]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [90-91]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [95-96]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [95-96]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(9)
        ClassicalDeclarationStmt [106-121]:
            symbol_id: 12
            ty_span: [106-110]
            init_expr: Expr [115-120]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Lt
                    lhs: Expr [115-116]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [115-116]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [119-120]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [119-120]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(9)
        ClassicalDeclarationStmt [130-146]:
            symbol_id: 13
            ty_span: [130-134]
            init_expr: Expr [139-145]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Lte
                    lhs: Expr [139-140]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [139-140]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [144-145]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [144-145]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(9)
        ClassicalDeclarationStmt [155-171]:
            symbol_id: 14
            ty_span: [155-159]
            init_expr: Expr [164-170]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Eq
                    lhs: Expr [164-165]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [164-165]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [169-170]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [169-170]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(9)
        ClassicalDeclarationStmt [180-196]:
            symbol_id: 15
            ty_span: [180-184]
            init_expr: Expr [189-195]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Neq
                    lhs: Expr [189-190]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [189-190]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [194-195]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [194-195]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(9)
    "#]]);
}

#[test]
fn bitarray_var_comparison_to_int_can_be_translated() {
    let input = r#"
        bit[1] x = "1";
        input int y;
        bool a = x > y;
        bool b = x >= y;
        bool c = x < y;
        bool d = x <= y;
        bool e = x == y;
        bool f = x != y;
        bool g = y > x;
        bool h = y >= x;
        bool i = y < x;
        bool j = y <= x;
        bool k = y == x;
        bool l = y != x;
    "#;

    check_stmt_kinds(input, &expect![[r#"
        ClassicalDeclarationStmt [9-24]:
            symbol_id: 8
            ty_span: [9-15]
            init_expr: Expr [20-23]:
                ty: BitArray(One(1), false)
                kind: Lit: Bitstring("1")
        InputDeclaration [33-45]:
            symbol_id: 9
        ClassicalDeclarationStmt [54-69]:
            symbol_id: 10
            ty_span: [54-58]
            init_expr: Expr [63-68]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Gt
                    lhs: Expr [63-64]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [63-64]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [67-68]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
        ClassicalDeclarationStmt [78-94]:
            symbol_id: 11
            ty_span: [78-82]
            init_expr: Expr [87-93]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Gte
                    lhs: Expr [87-88]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [87-88]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [92-93]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
        ClassicalDeclarationStmt [103-118]:
            symbol_id: 12
            ty_span: [103-107]
            init_expr: Expr [112-117]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Lt
                    lhs: Expr [112-113]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [112-113]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [116-117]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
        ClassicalDeclarationStmt [127-143]:
            symbol_id: 13
            ty_span: [127-131]
            init_expr: Expr [136-142]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Lte
                    lhs: Expr [136-137]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [136-137]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [141-142]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
        ClassicalDeclarationStmt [152-168]:
            symbol_id: 14
            ty_span: [152-156]
            init_expr: Expr [161-167]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Eq
                    lhs: Expr [161-162]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [161-162]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [166-167]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
        ClassicalDeclarationStmt [177-193]:
            symbol_id: 15
            ty_span: [177-181]
            init_expr: Expr [186-192]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Neq
                    lhs: Expr [186-187]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [186-187]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
                    rhs: Expr [191-192]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
        ClassicalDeclarationStmt [202-217]:
            symbol_id: 16
            ty_span: [202-206]
            init_expr: Expr [211-216]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Gt
                    lhs: Expr [211-212]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
                    rhs: Expr [215-216]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [215-216]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
        ClassicalDeclarationStmt [226-242]:
            symbol_id: 17
            ty_span: [226-230]
            init_expr: Expr [235-241]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Gte
                    lhs: Expr [235-236]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
                    rhs: Expr [240-241]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [240-241]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
        ClassicalDeclarationStmt [251-266]:
            symbol_id: 18
            ty_span: [251-255]
            init_expr: Expr [260-265]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Lt
                    lhs: Expr [260-261]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
                    rhs: Expr [264-265]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [264-265]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
        ClassicalDeclarationStmt [275-291]:
            symbol_id: 19
            ty_span: [275-279]
            init_expr: Expr [284-290]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Lte
                    lhs: Expr [284-285]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
                    rhs: Expr [289-290]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [289-290]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
        ClassicalDeclarationStmt [300-316]:
            symbol_id: 20
            ty_span: [300-304]
            init_expr: Expr [309-315]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Eq
                    lhs: Expr [309-310]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
                    rhs: Expr [314-315]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [314-315]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
        ClassicalDeclarationStmt [325-341]:
            symbol_id: 21
            ty_span: [325-329]
            init_expr: Expr [334-340]:
                ty: Bool(false)
                kind: BinaryOpExpr:
                    op: Neq
                    lhs: Expr [334-335]:
                        ty: Int(None, false)
                        kind: SymbolId(9)
                    rhs: Expr [339-340]:
                        ty: Int(None, false)
                        kind: Cast [0-0]:
                            ty: Int(None, false)
                            expr: Expr [339-340]:
                                ty: BitArray(One(1), false)
                                kind: SymbolId(8)
    "#]]);
}
