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

            Qasm.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
                ,-[Test.qasm:78:5]
             77 |     // ClassicalStmtInBox
             78 |     2;
                :     ^^
             79 | }
                `----

            Qasm.Lowerer.InvalidScope

              x break can only appear in loop scopes
                ,-[Test.qasm:82:1]
             81 | // InvalidScope break outside loop
             82 | break;
                : ^^^^^^
             83 | 
                `----

            Qasm.Lowerer.InvalidScope

              x continue can only appear in loop scopes
                ,-[Test.qasm:85:1]
             84 | // InvalidScope continue outside loop
             85 | continue;
                : ^^^^^^^^^
             86 | 
                `----

            Qasm.Lowerer.InvalidScope

              x return statements can only appear in subroutine scopes
                ,-[Test.qasm:88:1]
             87 | // InvalidScope return outside def
             88 | return;
                : ^^^^^^^
             89 | 
                `----

            Qasm.Lowerer.MissingTargetExpressionInReturnStmt

              x return statements on a non-void subroutine should have a target expression
                ,-[Test.qasm:92:5]
             91 | def missing_target_in_return() -> int {
             92 |     return;
                :     ^^^^^^^
             93 | }
                `----

            Qasm.Lowerer.ReturningExpressionFromVoidSubroutine

              x cannot return an expression from a void subroutine
                ,-[Test.qasm:97:12]
             96 | def returning_from_void_subroutine() {
             97 |     return 2;
                :            ^
             98 | }
                `----

            Qasm.Lowerer.ExprMustBeConst

              x const decl init expr must be a const expression
                 ,-[Test.qasm:108:23]
             107 | int non_const_val = 2;
             108 | const int const_val = non_const_val;
                 :                       ^^^^^^^^^^^^^
             109 | 
                 `----

              x array declarations are only allowed in global scope
                 ,-[Test.qasm:112:5]
             111 | {
             112 |     array[int, 1, 2] arr;
                 :     ^^^^^^^^^^^^^^^^^^^^^
             113 | }
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x def declarations must be done in global scope
                 ,-[Test.qasm:117:5]
             116 | {
             117 |     def f() {}
                 :     ^^^^^^^^^^
             118 | }
                 `----

            Qasm.Lowerer.GateDeclarationInNonGlobalScope

              x gate declarations must be done in global scope
                 ,-[Test.qasm:122:5]
             121 | {
             122 |     gate g q {}
                 :     ^^^^^^^^^^^
             123 | }
                 `----

            Qasm.Lowerer.QubitDeclarationInNonGlobalScope

              x qubit declarations must be done in global scope
                 ,-[Test.qasm:127:5]
             126 | {
             127 |     qubit non_global_qubit;
                 :     ^^^^^^^^^^^^^^^^^^^^^^^
             128 | }
                 `----

            Qasm.Lowerer.NonVoidDefShouldAlwaysReturn

              x non-void def should always return
                 ,-[Test.qasm:131:37]
             130 | // NonVoidDefShouldAlwaysReturn
             131 | def non_void_def_should_return() -> int {
                 :                                     ^^^
             132 | 
                 `----

            Qasm.Lowerer.ExternDeclarationInNonGlobalScope

              x extern declarations must be done in global scope
                 ,-[Test.qasm:141:5]
             140 | {
             141 |     extern f(int);
                 :     ^^^^^^^^^^^^^^
             142 | }
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 2 classical arguments, but 1 were provided
                 ,-[Test.qasm:146:1]
             145 | def invalid_arity_call(int a, int b) {}
             146 | invalid_arity_call(2);
                 : ^^^^^^^^^^^^^^^^^^^^^
             147 | 
                 `----

            Qasm.Lowerer.CannotCallNonFunction

              x cannot call an expression that is not a function
                 ,-[Test.qasm:149:1]
             148 | // CannotCallNonFunction
             149 | x(2);
                 : ^^^^
             150 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 1 classical arguments, but 2 were provided
                 ,-[Test.qasm:152:1]
             151 | // InvalidNumberOfClassicalArgs in gate call
             152 | rx(2.0, 3.0) q;
                 : ^^^^^^^^^^^^^^^
             153 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfQubitArgs

              x gate expects 1 qubit arguments, but 2 were provided
                 ,-[Test.qasm:155:1]
             154 | // InvalidNumberOfQubitArgs
             155 | rx(2.0) q, q;
                 : ^^^^^^^^^^^^^
             156 | 
                 `----

            Qasm.Lowerer.BroadcastCallQuantumArgsDisagreeInSize

              x first quantum register is of type qubit[1] but found an argument of type
              | qubit[2]
                 ,-[Test.qasm:158:18]
             157 | // BroadcastCallQuantumArgsDisagreeInSize
             158 | ryy(2.0) qreg_1, qreg_2;
                 :                  ^^^^^^
             159 | 
                 `----

            Qasm.Lowerer.ExprMustFitInU32

              x ctrl modifier argument must fit in a u32
                 ,-[Test.qasm:166:6]
             165 | // ExprMustFitInU32
             166 | ctrl(5000000000) @ x q;
                 :      ^^^^^^^^^^
             167 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:173:12]
             172 | // ArraySizeMustBeNonNegativeConstExpr
             173 | array[int, 2.0] non_int_array_size;
                 :            ^^^
             174 | 
                 `----

            Qasm.Lowerer.ExprMustBeNonNegativeInt

              x array size must be a non-negative integer
                 ,-[Test.qasm:176:12]
             175 | // ArraySizeMustBeNonNegativeConstExpr
             176 | array[int, -2] negative_array_size;
                 :            ^^
             177 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:179:12]
             178 | // DesignatorTooLarge
             179 | array[int, 5000000000] arr_size_too_large;
                 :            ^^^^^^^^^^
             180 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:182:5]
             181 | // TypeWidthMustBePositiveIntConstExpr
             182 | int[2.0] non_int_width;
                 :     ^^^
             183 | 
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:185:5]
             184 | // TypeWidthMustBePositiveIntConstExpr
             185 | int[0] zero_width;
                 :     ^
             186 | int[-2] negative_width;
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:186:5]
             185 | int[0] zero_width;
             186 | int[-2] negative_width;
                 :     ^^
             187 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:189:5]
             188 | // DesignatorTooLarge
             189 | int[5000000000] width_too_large;
                 :     ^^^^^^^^^^
             190 | 
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x float max width is 64 but 65 was provided
                 ,-[Test.qasm:192:1]
             191 | // TypeMaxWidthExceeded
             192 | float[65] float_width_too_large;
                 : ^^^^^^^^^
             193 | angle[65] angle_width_too_large;
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x angle max width is 64 but 65 was provided
                 ,-[Test.qasm:193:1]
             192 | float[65] float_width_too_large;
             193 | angle[65] angle_width_too_large;
                 : ^^^^^^^^^
             194 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type int
                 ,-[Test.qasm:196:1]
             195 | // Invalid literal cast in cast_expr_with_target_type_or_default(...)
             196 | int invalid_lit_cast = 2.0;
                 : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
             197 | 
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:205:5]
             204 | // QuantumTypesInBinaryExpression
             205 | 1 + q;
                 :     ^
             206 | q + 1;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:205:5]
             204 | // QuantumTypesInBinaryExpression
             205 | 1 + q;
                 :     ^
             206 | q + 1;
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:206:1]
             205 | 1 + q;
             206 | q + 1;
                 : ^
             207 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:206:1]
             205 | 1 + q;
             206 | q + 1;
                 : ^
             207 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type float
                 ,-[Test.qasm:210:1]
             209 | angle uncastable_to_int = 2.0;
             210 | uncastable_to_int + 3;
                 : ^^^^^^^^^^^^^^^^^
             211 | 3 + uncastable_to_int;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type const float
                 ,-[Test.qasm:211:5]
             210 | uncastable_to_int + 3;
             211 | 3 + uncastable_to_int;
                 :     ^^^^^^^^^^^^^^^^^
             212 | 
                 `----

            Qasm.Lowerer.OperatorNotAllowedForComplexValues

              x the operator OrB is not allowed for complex values
                 ,-[Test.qasm:214:1]
             213 | // OperatorNotAllowedForComplexValues
             214 | (2 + 1im) | 3im;
                 : ^^^^^^^^^^^^^^^
             215 | 
                 `----

            Qasm.Lowerer.IndexSetOnlyAllowedInAliasStmt

              x index sets are only allowed in alias statements
                 ,-[Test.qasm:217:8]
             216 | // IndexSetOnlyAllowedInAliasStmt
             217 | qreg_2[{0, 1}];
                 :        ^^^^^^
             218 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type const angle to type const int
                 ,-[Test.qasm:221:13]
             220 | array[int, 5] range_error;
             221 | range_error[const_uncastable_to_int:2.2];
                 :             ^^^^^^^^^^^^^^^^^^^^^^^
             222 | 
                 `----

            Qasm.Lowerer.ZeroStepInRange

              x range step cannot be zero
                 ,-[Test.qasm:224:13]
             223 | // ZeroStepInRange
             224 | range_error[1:0:3];
                 :             ^^^^^
             225 | 
                 `----

            Qasm.Lowerer.ZeroSizeArrayAccess

              x zero size array access is not allowed
                 ,-[Test.qasm:228:1]
             227 | array[int, 2, 0, 3] zero_size_array;
             228 | zero_size_array[1];
                 : ^^^^^^^^^^^^^^^^^^
             229 | 
                 `----
              help: array size must be a positive integer const expression

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type bit
                 ,-[Test.qasm:232:15]
             231 | bit non_indexable;
             232 | non_indexable[1];
                 :               ^
             233 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type qubit
                 ,-[Test.qasm:235:11]
             234 | // TooManyIndices
             235 | qreg_1[1, 2];
                 :           ^
             236 | 
                 `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: missing_symbol
                 ,-[Test.qasm:238:1]
             237 | // Missing symbol in lower_indexed_ident_expr(...)
             238 | missing_symbol[2];
                 : ^^^^^^^^^^^^^^
             239 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type unknown
                 ,-[Test.qasm:238:16]
             237 | // Missing symbol in lower_indexed_ident_expr(...)
             238 | missing_symbol[2];
                 :                ^
             239 | 
                 `----

            Qasm.Lowerer.EmptyIndexOperator

              x index operator must contain at least one index
                 ,-[Test.qasm:242:13]
             241 | bit[4] empty_index;
             242 | empty_index[];
                 :             ^
             243 | 
                 `----

            Qasm.Lowerer.ExternDeclarationCannotReturnDuration

              x extern declarations cannot return durations or stretches
                 ,-[Test.qasm:245:53]
             244 | // ExternDeclarationCannotReturnDuration
             245 | extern extern_function_with_duration_return(int) -> duration;
                 :                                                     ^^^^^^^^
             246 | 
                 `----

            Qasm.Lowerer.ExternDeclarationCannotReturnDuration

              x extern declarations cannot return durations or stretches
                 ,-[Test.qasm:248:52]
             247 | // ExternDeclarationCannotReturnDuration
             248 | extern extern_function_with_stretch_return(int) -> stretch;
                 :                                                    ^^^^^^^
             249 | 
                 `----

            Qasm.Lowerer.DefParameterCannotBeDuration

              x def parameters cannot be duration or stretch values
                 ,-[Test.qasm:251:43]
             250 | // DefParameterCannotBeDuration
             251 | def function_with_duration_param(qubit q, duration d) {
                 :                                           ^^^^^^^^
             252 |     delay[d] q;
                 `----

            Qasm.Lowerer.DurationMustBeKnownAtCompileTime

              x duration must be known at compile time
                 ,-[Test.qasm:252:11]
             251 | def function_with_duration_param(qubit q, duration d) {
             252 |     delay[d] q;
                 :           ^
             253 | }
                 `----

            Qasm.Lowerer.DefParameterCannotBeDuration

              x def parameters cannot be duration or stretch values
                 ,-[Test.qasm:256:49]
             255 | // DefParameterCannotBeDuration
             256 | def function_with_duration_array_param(qubit q, readonly array[duration, 2, 3] d) {
                 :                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             257 |     delay[d[0][0]] q;
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type readonly array[duration, 2, 3]
                 ,-[Test.qasm:257:11]
             256 | def function_with_duration_array_param(qubit q, readonly array[duration, 2, 3] d) {
             257 |     delay[d[0][0]] q;
                 :           ^
             258 | }
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type unknown
                 ,-[Test.qasm:257:16]
             256 | def function_with_duration_array_param(qubit q, readonly array[duration, 2, 3] d) {
             257 |     delay[d[0][0]] q;
                 :                ^
             258 | }
                 `----

            Qasm.Lowerer.ExprMustBeDuration

              x expression must be a duration
                 ,-[Test.qasm:257:11]
             256 | def function_with_duration_array_param(qubit q, readonly array[duration, 2, 3] d) {
             257 |     delay[d[0][0]] q;
                 :           ^^^
             258 | }
                 `----

            Qasm.Lowerer.DefParameterCannotBeDuration

              x def parameters cannot be duration or stretch values
                 ,-[Test.qasm:261:42]
             260 | // DefParameterCannotBeDuration
             261 | def function_with_stretch_param(qubit q, stretch d) {
                 :                                          ^^^^^^^
             262 |     delay[d] q;
                 `----

            Qasm.Lowerer.DurationMustBeKnownAtCompileTime

              x duration must be known at compile time
                 ,-[Test.qasm:262:11]
             261 | def function_with_stretch_param(qubit q, stretch d) {
             262 |     delay[d] q;
                 :           ^
             263 | }
                 `----
        "#]],
    );
}
