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

            Qasm.Lowerer.StdGateCalledButNotIncluded

              x A standard gate was called but not included
                ,-[Test.qasm:24:1]
             23 | // StdGateCalledButNotIncluded
             24 | x q;
                : ^^^^
             25 | 
                `----
              help: Did you mean to include the standard gate library 'stdgates.inc'?

            Qasm.Lowerer.RedefinedSymbol

              x redefined symbol: x
                ,-[Test.qasm:28:5]
             27 | include "stdgates.inc";
             28 | int x = 2;
                :     ^
             29 | 
                `----

            Qasm.Lowerer.RedefinedSymbol

              x redefined symbol: rxx
                ,-[Test.qasm:32:5]
             31 | rxx(2.0) q, q;
             32 | int rxx = 3;
                :     ^^^
             33 | 
                `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: undefined_symbol
                ,-[Test.qasm:35:1]
             34 | // UndefinedSymbol
             35 | undefined_symbol;
                : ^^^^^^^^^^^^^^^^
             36 | 
                `----

            Qasm.Lowerer.InconsistentTypesInAlias

              x inconsistent types in alias expression: qubit[2], bit[5]
                ,-[Test.qasm:40:15]
             39 | bit[5] alias_component_2;
             40 | let alias_1 = alias_component_1 ++ alias_component_2;
                :               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             41 | 
                `----

            Qasm.Lowerer.InvalidTypesInAlias

              x invalid types in alias expression: int
                ,-[Test.qasm:45:36]
             44 | int alias_component_4;
             45 | let alias_2 = alias_component_3 ++ alias_component_4;
                :                                    ^^^^^^^^^^^^^^^^^
             46 | 
                `----
              help: aliases can only be applied to quantum bits and registers

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_variable
                ,-[Test.qasm:49:1]
             48 | const int const_variable = 1;
             49 | const_variable = 2;
                : ^^^^^^^^^^^^^^
             50 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_bitarray
                ,-[Test.qasm:53:1]
             52 | const bit[2] const_bitarray = "11";
             53 | const_bitarray[1] = 0;
                : ^^^^^^^^^^^^^^
             54 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_variable
                ,-[Test.qasm:56:1]
             55 | // CannotUpdateConstVariable in simple assign_op
             56 | const_variable += 2;
                : ^^^^^^^^^^^^^^
             57 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_bitarray
                ,-[Test.qasm:59:1]
             58 | // CannotUpdateConstVariable in indexed assign_op
             59 | const_bitarray[1] += 7;
                : ^^^^^^^^^^^^^^
             60 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.ExprMustBeConst

              x undefined symbol: non_const_global_variable
                ,-[Test.qasm:64:13]
             63 | def try_capture_non_const_global_variable() {
             64 |     int a = non_const_global_variable;
                :             ^^^^^^^^^^^^^^^^^^^^^^^^^
             65 | }
                `----

            Qasm.Lowerer.ArrayDeclarationTypeError

              x expected an array of size 2 but found one of size 3
                ,-[Test.qasm:68:51]
             67 | // ArrayDeclarationTypeError
             68 | array[int, 2] array_initialized_with_wrong_size = {1, 2, 3};
                :                                                   ^^^^^^^^^
             69 | 
                `----

            Qasm.Lowerer.ArrayDeclarationTypeError

              x expected an array of size 2 but found Int(2)
                ,-[Test.qasm:71:54]
             70 | // ArrayDeclarationTypeError
             71 | array[int, 2] array_initialized_with_wrong_literal = 2;
                :                                                      ^
             72 | 
                `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const int to type array[int, 2]
                ,-[Test.qasm:71:1]
             70 | // ArrayDeclarationTypeError
             71 | array[int, 2] array_initialized_with_wrong_literal = 2;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             72 | 
                `----

            Qasm.Lowerer.NotSupported

              x string literals are not supported
                ,-[Test.qasm:74:1]
             73 | // NotSupported string literals
             74 | "string_literal";
                : ^^^^^^^^^^^^^^^^
             75 | 
                `----

            Qasm.Lowerer.TypeDoesNotSupportedUnaryNegation

              x unary negation is not allowed for instances of bool
                ,-[Test.qasm:79:2]
             78 | bool binary_negation_not_supported = true;
             79 | ~binary_negation_not_supported;
                :  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             80 | 
                `----

            Qasm.Lowerer.NotSupported

              x arrays with more than 7 dimensions are not supported
                ,-[Test.qasm:82:1]
             81 | // NotSupported arrays with more than 7 dimensions
             82 | array[int, 1, 2, 3, 1, 2, 3, 1, 2] array_with_more_than_7_dims;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             83 | 
                `----

            Qasm.Lowerer.NotSupported

              x stretch default values are not supported
                ,-[Test.qasm:85:1]
             84 | // NotSupported stretch default values
             85 | stretch stretch_val;
                : ^^^^^^^^^^^^^^^^^^^^
             86 | 
                `----

            Qasm.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
                ,-[Test.qasm:89:5]
             88 |     // ClassicalStmtInBox
             89 |     2;
                :     ^^
             90 | }
                `----

            Qasm.Lowerer.InvalidScope

              x break can only appear in loop scopes
                ,-[Test.qasm:93:1]
             92 | // InvalidScope break outside loop
             93 | break;
                : ^^^^^^
             94 | 
                `----

            Qasm.Lowerer.InvalidScope

              x continue can only appear in loop scopes
                ,-[Test.qasm:96:1]
             95 | // InvalidScope continue outside loop
             96 | continue;
                : ^^^^^^^^^
             97 | 
                `----

            Qasm.Lowerer.InvalidScope

              x return statements can only appear in subroutine scopes
                 ,-[Test.qasm:99:1]
              98 | // InvalidScope return outside def
              99 | return;
                 : ^^^^^^^
             100 | 
                 `----

            Qasm.Lowerer.MissingTargetExpressionInReturnStmt

              x return statements on a non-void subroutine should have a target expression
                 ,-[Test.qasm:103:5]
             102 | def missing_target_in_return() -> int {
             103 |     return;
                 :     ^^^^^^^
             104 | }
                 `----

            Qasm.Lowerer.ReturningExpressionFromVoidSubroutine

              x cannot return an expression from a void subroutine
                 ,-[Test.qasm:108:12]
             107 | def returning_from_void_subroutine() {
             108 |     return 2;
                 :            ^
             109 | }
                 `----

            Qasm.Lowerer.ExprMustBeConst

              x const decl init expr must be a const expression
                 ,-[Test.qasm:119:23]
             118 | int non_const_val = 2;
             119 | const int const_val = non_const_val;
                 :                       ^^^^^^^^^^^^^
             120 | 
                 `----

              x array declarations are only allowed in global scope
                 ,-[Test.qasm:123:5]
             122 | {
             123 |     array[int, 1, 2] arr;
                 :     ^^^^^^^^^^^^^^^^^^^^^
             124 | }
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x def declarations must be done in global scope
                 ,-[Test.qasm:128:5]
             127 | {
             128 |     def f() {}
                 :     ^^^^^^^^^^
             129 | }
                 `----

            Qasm.Lowerer.GateDeclarationInNonGlobalScope

              x gate declarations must be done in global scope
                 ,-[Test.qasm:133:5]
             132 | {
             133 |     gate g q {}
                 :     ^^^^^^^^^^^
             134 | }
                 `----

            Qasm.Lowerer.QubitDeclarationInNonGlobalScope

              x qubit declarations must be done in global scope
                 ,-[Test.qasm:138:5]
             137 | {
             138 |     qubit non_global_qubit;
                 :     ^^^^^^^^^^^^^^^^^^^^^^^
             139 | }
                 `----

            Qasm.Lowerer.NonVoidDefShouldAlwaysReturn

              x non-void def should always return
                 ,-[Test.qasm:142:37]
             141 | // NonVoidDefShouldAlwaysReturn
             142 | def non_void_def_should_return() -> int {
                 :                                     ^^^
             143 | 
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x extern declarations must be done in global scope
                 ,-[Test.qasm:152:5]
             151 | {
             152 |     extern f(int);
                 :     ^^^^^^^^^^^^^^
             153 | }
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 2 classical arguments, but 1 were provided
                 ,-[Test.qasm:157:1]
             156 | def invalid_arity_call(int a, int b) {}
             157 | invalid_arity_call(2);
                 : ^^^^^^^^^^^^^^^^^^^^^
             158 | 
                 `----

            Qasm.Lowerer.GateCalledLikeFunc

              x gate called like function: gate(0, 1)
                 ,-[Test.qasm:160:1]
             159 | // CannotCallNonFunction
             160 | x(2);
                 : ^^^^
             161 | 
                 `----
              help: ensure that qubit arguments are provided to the gate call

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 1 classical arguments, but 2 were provided
                 ,-[Test.qasm:163:1]
             162 | // InvalidNumberOfClassicalArgs in gate call
             163 | rx(2.0, 3.0) q;
                 : ^^^^^^^^^^^^^^^
             164 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfQubitArgs

              x gate expects 1 qubit arguments, but 2 were provided
                 ,-[Test.qasm:166:1]
             165 | // InvalidNumberOfQubitArgs
             166 | rx(2.0) q, q;
                 : ^^^^^^^^^^^^^
             167 | 
                 `----

            Qasm.Lowerer.BroadcastCallQuantumArgsDisagreeInSize

              x first quantum register is of type qubit[1] but found an argument of type
              | qubit[2]
                 ,-[Test.qasm:169:18]
             168 | // BroadcastCallQuantumArgsDisagreeInSize
             169 | ryy(2.0) qreg_1, qreg_2;
                 :                  ^^^^^^
             170 | 
                 `----

            Qasm.Lowerer.ExprMustFitInU32

              x ctrl modifier argument must fit in a u32
                 ,-[Test.qasm:177:6]
             176 | // ExprMustFitInU32
             177 | ctrl(5000000000) @ x q;
                 :      ^^^^^^^^^^
             178 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:184:12]
             183 | // ArraySizeMustBeNonNegativeConstExpr
             184 | array[int, 2.0] non_int_array_size;
                 :            ^^^
             185 | 
                 `----

            Qasm.Lowerer.ExprMustBeNonNegativeInt

              x array size must be a non-negative integer
                 ,-[Test.qasm:187:12]
             186 | // ArraySizeMustBeNonNegativeConstExpr
             187 | array[int, -2] negative_array_size;
                 :            ^^
             188 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:190:12]
             189 | // DesignatorTooLarge
             190 | array[int, 5000000000] arr_size_too_large;
                 :            ^^^^^^^^^^
             191 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:193:5]
             192 | // TypeWidthMustBePositiveIntConstExpr
             193 | int[2.0] non_int_width;
                 :     ^^^
             194 | 
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:196:5]
             195 | // TypeWidthMustBePositiveIntConstExpr
             196 | int[0] zero_width;
                 :     ^
             197 | int[-2] negative_width;
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:197:5]
             196 | int[0] zero_width;
             197 | int[-2] negative_width;
                 :     ^^
             198 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:200:5]
             199 | // DesignatorTooLarge
             200 | int[5000000000] width_too_large;
                 :     ^^^^^^^^^^
             201 | 
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x float max width is 64 but 65 was provided
                 ,-[Test.qasm:203:1]
             202 | // TypeMaxWidthExceeded
             203 | float[65] float_width_too_large;
                 : ^^^^^^^^^
             204 | angle[65] angle_width_too_large;
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x angle max width is 64 but 65 was provided
                 ,-[Test.qasm:204:1]
             203 | float[65] float_width_too_large;
             204 | angle[65] angle_width_too_large;
                 : ^^^^^^^^^
             205 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type int
                 ,-[Test.qasm:207:1]
             206 | // Invalid literal cast in cast_expr_with_target_type_or_default(...)
             207 | int invalid_lit_cast = 2.0;
                 : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
             208 | 
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:216:5]
             215 | // QuantumTypesInBinaryExpression
             216 | 1 + q;
                 :     ^
             217 | q + 1;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:216:5]
             215 | // QuantumTypesInBinaryExpression
             216 | 1 + q;
                 :     ^
             217 | q + 1;
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:217:1]
             216 | 1 + q;
             217 | q + 1;
                 : ^
             218 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:217:1]
             216 | 1 + q;
             217 | q + 1;
                 : ^
             218 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type float
                 ,-[Test.qasm:221:1]
             220 | angle uncastable_to_int = 2.0;
             221 | uncastable_to_int + 3;
                 : ^^^^^^^^^^^^^^^^^
             222 | 3 + uncastable_to_int;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type const float
                 ,-[Test.qasm:222:5]
             221 | uncastable_to_int + 3;
             222 | 3 + uncastable_to_int;
                 :     ^^^^^^^^^^^^^^^^^
             223 | 
                 `----

            Qasm.Lowerer.OperatorNotAllowedForComplexValues

              x the operator OrB is not allowed for complex values
                 ,-[Test.qasm:225:1]
             224 | // OperatorNotAllowedForComplexValues
             225 | (2 + 1im) | 3im;
                 : ^^^^^^^^^^^^^^^
             226 | 
                 `----

            Qasm.Lowerer.IndexSetOnlyAllowedInAliasStmt

              x index sets are only allowed in alias statements
                 ,-[Test.qasm:228:8]
             227 | // IndexSetOnlyAllowedInAliasStmt
             228 | qreg_2[{0, 1}];
                 :        ^^^^^^
             229 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type const angle to type const int
                 ,-[Test.qasm:232:13]
             231 | array[int, 5] range_error;
             232 | range_error[const_uncastable_to_int:2.2];
                 :             ^^^^^^^^^^^^^^^^^^^^^^^
             233 | 
                 `----

            Qasm.Lowerer.ZeroStepInRange

              x range step cannot be zero
                 ,-[Test.qasm:235:13]
             234 | // ZeroStepInRange
             235 | range_error[1:0:3];
                 :             ^^^^^
             236 | 
                 `----

            Qasm.Lowerer.ZeroSizeArrayAccess

              x zero size array access is not allowed
                 ,-[Test.qasm:239:1]
             238 | array[int, 2, 0, 3] zero_size_array;
             239 | zero_size_array[1];
                 : ^^^^^^^^^^^^^^^^^^
             240 | 
                 `----
              help: array size must be a positive integer const expression

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type bit
                 ,-[Test.qasm:243:15]
             242 | bit non_indexable;
             243 | non_indexable[1];
                 :               ^
             244 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type qubit
                 ,-[Test.qasm:246:11]
             245 | // TooManyIndices
             246 | qreg_1[1, 2];
                 :           ^
             247 | 
                 `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: missing_symbol
                 ,-[Test.qasm:249:1]
             248 | // Missing symbol in lower_indexed_ident_expr(...)
             249 | missing_symbol[2];
                 : ^^^^^^^^^^^^^^
             250 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type unknown
                 ,-[Test.qasm:249:16]
             248 | // Missing symbol in lower_indexed_ident_expr(...)
             249 | missing_symbol[2];
                 :                ^
             250 | 
                 `----

            Qasm.Lowerer.EmptyIndexOperator

              x index operator must contain at least one index
                 ,-[Test.qasm:253:13]
             252 | bit[4] empty_index;
             253 | empty_index[];
                 :             ^
             254 | 
                 `----

            Qasm.Lowerer.CannotCallNonFunction

              x cannot call an expression that is not a function
                 ,-[Test.qasm:256:1]
             255 | // CannotCallNonFunction
             256 | empty_index();
                 : ^^^^^^^^^^^^^
             257 | 
                 `----

            Qasm.Lowerer.FuncCalledLikeGate

              x function called like gate: def (qubit) -> void
                 ,-[Test.qasm:259:5]
             258 | // FuncCalledLikeGate
             259 | def func_called_like_gate(qubit q) {}
                 :     ^^^^^^^^^^^^^^^^^^^^^
             260 | func_called_like_gate q;
                 `----
              help: function parameters must be in parentheses

            Qasm.Lowerer.GateCallMissingParams

              x gate call missing parameters: gate(0, 1)
                 ,-[Test.qasm:263:1]
             262 | // GateCallMissingParams
             263 | h;
                 : ^
             264 | 
                 `----
              help: ensure that any classical and quantum arguments are provided to the
                    gate call

            Qasm.Lowerer.FuncMissingParams

              x function call missing parameters: def (qubit) -> void
                 ,-[Test.qasm:267:1]
             266 | // FuncMissingParams
             267 | func_called_like_gate;
                 : ^^^^^^^^^^^^^^^^^^^^^
                 `----
              help: a function call must use parentheses, with any parameters inside
                    those parentheses.
        "#]],
    );
}
