// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp;
use expect_test::expect;

const SOURCE: &str = include_str!("../resources/openqasm_lowerer_errors_test.qasm");

#[allow(clippy::too_many_lines)]
#[test]
fn check_lowerer_error_spans_are_correct() {
    check_qasm_to_qsharp(
        SOURCE,
        &expect![[r#"
              x Not Found: Could not resolve include file: missing_file
                ,-[Test.qasm:21:1]
             20 | // lowerer, so that we can contruct the error with the right span.
             21 | include "missing_file";
                : ^^^^^^^^^^^^^^^^^^^^^^^
             22 | 
                `----

            Qasm.Lowerer.UnsupportedVersion

              x unsupported version: '4.0'
               ,-[Test.qasm:4:10]
             3 | // UnsupportedVersion
             4 | OPENQASM 4.0;
               :          ^^^
             5 | 
               `----

            Qasm.Lowerer.IncludeNotInGlobalScope

              x include stdgates.inc must be declared in global scope
                ,-[Test.qasm:16:5]
             15 | {
             16 |     include "stdgates.inc";
                :     ^^^^^^^^^^^^^^^^^^^^^^^
             17 | }
                `----

            Qasm.Lowerer.RedefinedSymbol

              x redefined symbol: x
                ,-[Test.qasm:25:5]
             24 | include "stdgates.inc";
             25 | int x = 2;
                :     ^
             26 | 
                `----

            Qasm.Lowerer.RedefinedSymbol

              x redefined symbol: rxx
                ,-[Test.qasm:29:5]
             28 | rxx(2.0) q, q;
             29 | int rxx = 3;
                :     ^^^
             30 | 
                `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: undefined_symbol
                ,-[Test.qasm:32:1]
             31 | // UndefinedSymbol
             32 | undefined_symbol;
                : ^^^^^^^^^^^^^^^^
             33 | 
                `----

            Qasm.Lowerer.InconsistentTypesInAlias

              x inconsistent types in alias expression: Expr [842-859]:
              |     ty: array[int, 2]
              |     kind: SymbolId(45), Expr [863-880]:
              |     ty: array[angle, 2]
              |     kind: SymbolId(46)
                ,-[Test.qasm:37:1]
             36 | array[angle, 2] alias_component_2 = {1.0, 2.0};
             37 | let alias = alias_component_1 ++ alias_component_2;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             38 | 
                `----

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_variable
                ,-[Test.qasm:41:1]
             40 | const int const_variable = 1;
             41 | const_variable = 2;
                : ^^^^^^^^^^^^^^
             42 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_bitarray
                ,-[Test.qasm:45:1]
             44 | const bit[2] const_bitarray = "11";
             45 | const_bitarray[1] = 0;
                : ^^^^^^^^^^^^^^
             46 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_variable
                ,-[Test.qasm:48:1]
             47 | // CannotUpdateConstVariable in simple assign_op
             48 | const_variable += 2;
                : ^^^^^^^^^^^^^^
             49 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_bitarray
                ,-[Test.qasm:51:1]
             50 | // CannotUpdateConstVariable in indexed assign_op
             51 | const_bitarray[1] += 7;
                : ^^^^^^^^^^^^^^
             52 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: non_const_global_variable
                ,-[Test.qasm:56:13]
             55 | def try_capture_non_const_global_variable() {
             56 |     int a = non_const_global_variable;
                :             ^^^^^^^^^^^^^^^^^^^^^^^^^
             57 | }
                `----

            Qasm.Lowerer.ArrayDeclarationTypeError

              x expected an array of size 2 but found one of size 3
                ,-[Test.qasm:60:51]
             59 | // ArrayDeclarationTypeError
             60 | array[int, 2] array_initialized_with_wrong_size = {1, 2, 3};
                :                                                   ^^^^^^^^^
             61 | 
                `----

            Qasm.Lowerer.ArrayDeclarationTypeError

              x expected an array of size 2 but found Int(2)
                ,-[Test.qasm:63:54]
             62 | // ArrayDeclarationTypeError
             63 | array[int, 2] array_initialized_with_wrong_literal = 2;
                :                                                      ^
             64 | 
                `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const int to type array[int, 2]
                ,-[Test.qasm:63:1]
             62 | // ArrayDeclarationTypeError
             63 | array[int, 2] array_initialized_with_wrong_literal = 2;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             64 | 
                `----

            Qasm.Lowerer.NotSupported

              x string literals are not supported
                ,-[Test.qasm:66:1]
             65 | // NotSupported string literals
             66 | "string_literal";
                : ^^^^^^^^^^^^^^^^
             67 | 
                `----

            Qasm.Lowerer.TypeDoesNotSupportedUnaryNegation

              x unary negation is not allowed for instances of bool
                ,-[Test.qasm:71:2]
             70 | bool binary_negation_not_supported = true;
             71 | ~binary_negation_not_supported;
                :  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             72 | 
                `----

            Qasm.Lowerer.NotSupported

              x arrays with more than 7 dimensions are not supported
                ,-[Test.qasm:74:1]
             73 | // NotSupported arrays with more than 7 dimensions
             74 | array[int, 1, 2, 3, 1, 2, 3, 1, 2] array_with_more_than_7_dims;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             75 | 
                `----

            Qasm.Lowerer.NotSupported

              x stretch default values are not supported
                ,-[Test.qasm:77:1]
             76 | // NotSupported stretch default values
             77 | stretch stretch_val;
                : ^^^^^^^^^^^^^^^^^^^^
             78 | 
                `----

            Qasm.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
                ,-[Test.qasm:81:5]
             80 |     // ClassicalStmtInBox
             81 |     2;
                :     ^^
             82 | }
                `----

            Qasm.Lowerer.InvalidScope

              x break can only appear in loop scopes
                ,-[Test.qasm:85:1]
             84 | // InvalidScope break outside loop
             85 | break;
                : ^^^^^^
             86 | 
                `----

            Qasm.Lowerer.InvalidScope

              x continue can only appear in loop scopes
                ,-[Test.qasm:88:1]
             87 | // InvalidScope continue outside loop
             88 | continue;
                : ^^^^^^^^^
             89 | 
                `----

            Qasm.Lowerer.InvalidScope

              x return statements can only appear in subroutine scopes
                ,-[Test.qasm:91:1]
             90 | // InvalidScope return outside def
             91 | return;
                : ^^^^^^^
             92 | 
                `----

            Qasm.Lowerer.MissingTargetExpressionInReturnStmt

              x return statements on a non-void subroutine should have a target expression
                ,-[Test.qasm:95:5]
             94 | def missing_target_in_return() -> int {
             95 |     return;
                :     ^^^^^^^
             96 | }
                `----

            Qasm.Lowerer.ReturningExpressionFromVoidSubroutine

              x cannot return an expression from a void subroutine
                 ,-[Test.qasm:100:12]
              99 | def returning_from_void_subroutine() {
             100 |     return 2;
                 :            ^
             101 | }
                 `----

            Qasm.Lowerer.ExprMustBeConst

              x const decl init expr must be a const expression
                 ,-[Test.qasm:111:23]
             110 | int non_const_val = 2;
             111 | const int const_val = non_const_val;
                 :                       ^^^^^^^^^^^^^
             112 | 
                 `----

              x array declarations are only allowed in global scope
                 ,-[Test.qasm:115:5]
             114 | {
             115 |     array[int, 1, 2] arr;
                 :     ^^^^^^^^^^^^^^^^^^^^^
             116 | }
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x def declarations must be done in global scope
                 ,-[Test.qasm:120:5]
             119 | {
             120 |     def f() {}
                 :     ^^^^^^^^^^
             121 | }
                 `----

            Qasm.Lowerer.GateDeclarationInNonGlobalScope

              x gate declarations must be done in global scope
                 ,-[Test.qasm:125:5]
             124 | {
             125 |     gate g q {}
                 :     ^^^^^^^^^^^
             126 | }
                 `----

            Qasm.Lowerer.QubitDeclarationInNonGlobalScope

              x qubit declarations must be done in global scope
                 ,-[Test.qasm:130:5]
             129 | {
             130 |     qubit non_global_qubit;
                 :     ^^^^^^^^^^^^^^^^^^^^^^^
             131 | }
                 `----

            Qasm.Lowerer.NonVoidDefShouldAlwaysReturn

              x non-void def should always return
                 ,-[Test.qasm:134:37]
             133 | // NonVoidDefShouldAlwaysReturn
             134 | def non_void_def_should_return() -> int {
                 :                                     ^^^
             135 | 
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x extern declarations must be done in global scope
                 ,-[Test.qasm:144:5]
             143 | {
             144 |     extern f(int);
                 :     ^^^^^^^^^^^^^^
             145 | }
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 2 classical arguments, but 1 were provided
                 ,-[Test.qasm:149:1]
             148 | def invalid_arity_call(int a, int b) {}
             149 | invalid_arity_call(2);
                 : ^^^^^^^^^^^^^^^^^^^^^
             150 | 
                 `----

            Qasm.Lowerer.CannotCallNonFunction

              x cannot call an expression that is not a function
                 ,-[Test.qasm:152:1]
             151 | // CannotCallNonFunction
             152 | x(2);
                 : ^^^^
             153 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 1 classical arguments, but 2 were provided
                 ,-[Test.qasm:155:1]
             154 | // InvalidNumberOfClassicalArgs in gate call
             155 | rx(2.0, 3.0) q;
                 : ^^^^^^^^^^^^^^^
             156 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfQubitArgs

              x gate expects 1 qubit arguments, but 2 were provided
                 ,-[Test.qasm:158:1]
             157 | // InvalidNumberOfQubitArgs
             158 | rx(2.0) q, q;
                 : ^^^^^^^^^^^^^
             159 | 
                 `----

            Qasm.Lowerer.BroadcastCallQuantumArgsDisagreeInSize

              x first quantum register is of type qubit[1] but found an argument of type
              | qubit[2]
                 ,-[Test.qasm:161:18]
             160 | // BroadcastCallQuantumArgsDisagreeInSize
             161 | ryy(2.0) qreg_1, qreg_2;
                 :                  ^^^^^^
             162 | 
                 `----

            Qasm.Lowerer.ExprMustFitInU32

              x ctrl modifier argument must fit in a u32
                 ,-[Test.qasm:169:6]
             168 | // ExprMustFitInU32
             169 | ctrl(5000000000) @ x q;
                 :      ^^^^^^^^^^
             170 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:176:12]
             175 | // ArraySizeMustBeNonNegativeConstExpr
             176 | array[int, 2.0] non_int_array_size;
                 :            ^^^
             177 | 
                 `----

            Qasm.Lowerer.ExprMustBeNonNegativeInt

              x array size must be a non-negative integer
                 ,-[Test.qasm:179:12]
             178 | // ArraySizeMustBeNonNegativeConstExpr
             179 | array[int, -2] negative_array_size;
                 :            ^^
             180 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:182:12]
             181 | // DesignatorTooLarge
             182 | array[int, 5000000000] arr_size_too_large;
                 :            ^^^^^^^^^^
             183 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:185:5]
             184 | // TypeWidthMustBePositiveIntConstExpr
             185 | int[2.0] non_int_width;
                 :     ^^^
             186 | 
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:188:5]
             187 | // TypeWidthMustBePositiveIntConstExpr
             188 | int[0] zero_width;
                 :     ^
             189 | int[-2] negative_width;
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:189:5]
             188 | int[0] zero_width;
             189 | int[-2] negative_width;
                 :     ^^
             190 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:192:5]
             191 | // DesignatorTooLarge
             192 | int[5000000000] width_too_large;
                 :     ^^^^^^^^^^
             193 | 
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x float max width is 64 but 65 was provided
                 ,-[Test.qasm:195:1]
             194 | // TypeMaxWidthExceeded
             195 | float[65] float_width_too_large;
                 : ^^^^^^^^^
             196 | angle[65] angle_width_too_large;
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x angle max width is 64 but 65 was provided
                 ,-[Test.qasm:196:1]
             195 | float[65] float_width_too_large;
             196 | angle[65] angle_width_too_large;
                 : ^^^^^^^^^
             197 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type int
                 ,-[Test.qasm:199:1]
             198 | // Invalid literal cast in cast_expr_with_target_type_or_default(...)
             199 | int invalid_lit_cast = 2.0;
                 : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
             200 | 
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:208:5]
             207 | // QuantumTypesInBinaryExpression
             208 | 1 + q;
                 :     ^
             209 | q + 1;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:208:5]
             207 | // QuantumTypesInBinaryExpression
             208 | 1 + q;
                 :     ^
             209 | q + 1;
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:209:1]
             208 | 1 + q;
             209 | q + 1;
                 : ^
             210 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:209:1]
             208 | 1 + q;
             209 | q + 1;
                 : ^
             210 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type float
                 ,-[Test.qasm:213:1]
             212 | angle uncastable_to_int = 2.0;
             213 | uncastable_to_int + 3;
                 : ^^^^^^^^^^^^^^^^^
             214 | 3 + uncastable_to_int;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type const float
                 ,-[Test.qasm:214:5]
             213 | uncastable_to_int + 3;
             214 | 3 + uncastable_to_int;
                 :     ^^^^^^^^^^^^^^^^^
             215 | 
                 `----

            Qasm.Lowerer.OperatorNotAllowedForComplexValues

              x the operator OrB is not allowed for complex values
                 ,-[Test.qasm:217:1]
             216 | // OperatorNotAllowedForComplexValues
             217 | (2 + 1im) | 3im;
                 : ^^^^^^^^^^^^^^^
             218 | 
                 `----

            Qasm.Lowerer.IndexSetOnlyAllowedInAliasStmt

              x index sets are only allowed in alias statements
                 ,-[Test.qasm:220:8]
             219 | // IndexSetOnlyAllowedInAliasStmt
             220 | qreg_2[{0, 1}];
                 :        ^^^^^^
             221 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type const angle to type const int
                 ,-[Test.qasm:224:13]
             223 | array[int, 5] range_error;
             224 | range_error[const_uncastable_to_int:2.2];
                 :             ^^^^^^^^^^^^^^^^^^^^^^^
             225 | 
                 `----

            Qasm.Lowerer.ZeroStepInRange

              x range step cannot be zero
                 ,-[Test.qasm:227:13]
             226 | // ZeroStepInRange
             227 | range_error[1:0:3];
                 :             ^^^^^
             228 | 
                 `----

            Qasm.Lowerer.ZeroSizeArrayAccess

              x zero size array access is not allowed
                 ,-[Test.qasm:231:1]
             230 | array[int, 2, 0, 3] zero_size_array;
             231 | zero_size_array[1];
                 : ^^^^^^^^^^^^^^^^^^
             232 | 
                 `----
              help: array size must be a positive integer const expression

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type bit
                 ,-[Test.qasm:235:15]
             234 | bit non_indexable;
             235 | non_indexable[1];
                 :               ^
             236 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type qubit
                 ,-[Test.qasm:238:11]
             237 | // TooManyIndices
             238 | qreg_1[1, 2];
                 :           ^
             239 | 
                 `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: missing_symbol
                 ,-[Test.qasm:241:1]
             240 | // Missing symbol in lower_indexed_ident_expr(...)
             241 | missing_symbol[2];
                 : ^^^^^^^^^^^^^^
             242 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type unknown
                 ,-[Test.qasm:241:16]
             240 | // Missing symbol in lower_indexed_ident_expr(...)
             241 | missing_symbol[2];
                 :                ^
             242 | 
                 `----

            Qasm.Lowerer.EmptyIndexOperator

              x index operator must contain at least one index
                 ,-[Test.qasm:245:13]
             244 | bit[4] empty_index;
             245 | empty_index[];
                 :             ^
                 `----
        "#]],
    );
}
