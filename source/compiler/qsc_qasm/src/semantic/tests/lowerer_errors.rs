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

              x inconsistent types in alias expression: Expr [879-896]:
              |     ty: array[int, 2]
              |     kind: SymbolId(45), Expr [900-917]:
              |     ty: array[angle, 2]
              |     kind: SymbolId(46)
                ,-[Test.qasm:40:1]
             39 | array[angle, 2] alias_component_2 = {1.0, 2.0};
             40 | let alias = alias_component_1 ++ alias_component_2;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             41 | 
                `----

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_variable
                ,-[Test.qasm:44:1]
             43 | const int const_variable = 1;
             44 | const_variable = 2;
                : ^^^^^^^^^^^^^^
             45 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_bitarray
                ,-[Test.qasm:48:1]
             47 | const bit[2] const_bitarray = "11";
             48 | const_bitarray[1] = 0;
                : ^^^^^^^^^^^^^^
             49 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_variable
                ,-[Test.qasm:51:1]
             50 | // CannotUpdateConstVariable in simple assign_op
             51 | const_variable += 2;
                : ^^^^^^^^^^^^^^
             52 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.CannotUpdateConstVariable

              x cannot update const variable const_bitarray
                ,-[Test.qasm:54:1]
             53 | // CannotUpdateConstVariable in indexed assign_op
             54 | const_bitarray[1] += 7;
                : ^^^^^^^^^^^^^^
             55 | 
                `----
              help: mutable variables must be declared without the keyword `const`

            Qasm.Lowerer.ExprMustBeConst

              x a captured variable must be a const expression
                ,-[Test.qasm:59:13]
             58 | def try_capture_non_const_global_variable() {
             59 |     int a = non_const_global_variable;
                :             ^^^^^^^^^^^^^^^^^^^^^^^^^
             60 | }
                `----

            Qasm.Lowerer.ArrayDeclarationTypeError

              x expected an array of size 2 but found one of size 3
                ,-[Test.qasm:63:51]
             62 | // ArrayDeclarationTypeError
             63 | array[int, 2] array_initialized_with_wrong_size = {1, 2, 3};
                :                                                   ^^^^^^^^^
             64 | 
                `----

            Qasm.Lowerer.ArrayDeclarationTypeError

              x expected an array of size 2 but found Int(2)
                ,-[Test.qasm:66:54]
             65 | // ArrayDeclarationTypeError
             66 | array[int, 2] array_initialized_with_wrong_literal = 2;
                :                                                      ^
             67 | 
                `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const int to type array[int, 2]
                ,-[Test.qasm:66:1]
             65 | // ArrayDeclarationTypeError
             66 | array[int, 2] array_initialized_with_wrong_literal = 2;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             67 | 
                `----

            Qasm.Lowerer.NotSupported

              x string literals are not supported
                ,-[Test.qasm:69:1]
             68 | // NotSupported string literals
             69 | "string_literal";
                : ^^^^^^^^^^^^^^^^
             70 | 
                `----

            Qasm.Lowerer.TypeDoesNotSupportedUnaryNegation

              x unary negation is not allowed for instances of bool
                ,-[Test.qasm:74:2]
             73 | bool binary_negation_not_supported = true;
             74 | ~binary_negation_not_supported;
                :  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             75 | 
                `----

            Qasm.Lowerer.NotSupported

              x arrays with more than 7 dimensions are not supported
                ,-[Test.qasm:77:1]
             76 | // NotSupported arrays with more than 7 dimensions
             77 | array[int, 1, 2, 3, 1, 2, 3, 1, 2] array_with_more_than_7_dims;
                : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             78 | 
                `----

            Qasm.Lowerer.NotSupported

              x stretch default values are not supported
                ,-[Test.qasm:80:1]
             79 | // NotSupported stretch default values
             80 | stretch stretch_val;
                : ^^^^^^^^^^^^^^^^^^^^
             81 | 
                `----

            Qasm.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
                ,-[Test.qasm:84:5]
             83 |     // ClassicalStmtInBox
             84 |     2;
                :     ^^
             85 | }
                `----

            Qasm.Lowerer.InvalidScope

              x break can only appear in loop scopes
                ,-[Test.qasm:88:1]
             87 | // InvalidScope break outside loop
             88 | break;
                : ^^^^^^
             89 | 
                `----

            Qasm.Lowerer.InvalidScope

              x continue can only appear in loop scopes
                ,-[Test.qasm:91:1]
             90 | // InvalidScope continue outside loop
             91 | continue;
                : ^^^^^^^^^
             92 | 
                `----

            Qasm.Lowerer.InvalidScope

              x return statements can only appear in subroutine scopes
                ,-[Test.qasm:94:1]
             93 | // InvalidScope return outside def
             94 | return;
                : ^^^^^^^
             95 | 
                `----

            Qasm.Lowerer.MissingTargetExpressionInReturnStmt

              x return statements on a non-void subroutine should have a target expression
                ,-[Test.qasm:98:5]
             97 | def missing_target_in_return() -> int {
             98 |     return;
                :     ^^^^^^^
             99 | }
                `----

            Qasm.Lowerer.ReturningExpressionFromVoidSubroutine

              x cannot return an expression from a void subroutine
                 ,-[Test.qasm:103:12]
             102 | def returning_from_void_subroutine() {
             103 |     return 2;
                 :            ^
             104 | }
                 `----

            Qasm.Lowerer.ExprMustBeConst

              x const decl init expr must be a const expression
                 ,-[Test.qasm:114:23]
             113 | int non_const_val = 2;
             114 | const int const_val = non_const_val;
                 :                       ^^^^^^^^^^^^^
             115 | 
                 `----

              x array declarations are only allowed in global scope
                 ,-[Test.qasm:118:5]
             117 | {
             118 |     array[int, 1, 2] arr;
                 :     ^^^^^^^^^^^^^^^^^^^^^
             119 | }
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x def declarations must be done in global scope
                 ,-[Test.qasm:123:5]
             122 | {
             123 |     def f() {}
                 :     ^^^^^^^^^^
             124 | }
                 `----

            Qasm.Lowerer.GateDeclarationInNonGlobalScope

              x gate declarations must be done in global scope
                 ,-[Test.qasm:128:5]
             127 | {
             128 |     gate g q {}
                 :     ^^^^^^^^^^^
             129 | }
                 `----

            Qasm.Lowerer.QubitDeclarationInNonGlobalScope

              x qubit declarations must be done in global scope
                 ,-[Test.qasm:133:5]
             132 | {
             133 |     qubit non_global_qubit;
                 :     ^^^^^^^^^^^^^^^^^^^^^^^
             134 | }
                 `----

            Qasm.Lowerer.NonVoidDefShouldAlwaysReturn

              x non-void def should always return
                 ,-[Test.qasm:137:37]
             136 | // NonVoidDefShouldAlwaysReturn
             137 | def non_void_def_should_return() -> int {
                 :                                     ^^^
             138 | 
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x extern declarations must be done in global scope
                 ,-[Test.qasm:147:5]
             146 | {
             147 |     extern f(int);
                 :     ^^^^^^^^^^^^^^
             148 | }
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 2 classical arguments, but 1 were provided
                 ,-[Test.qasm:152:1]
             151 | def invalid_arity_call(int a, int b) {}
             152 | invalid_arity_call(2);
                 : ^^^^^^^^^^^^^^^^^^^^^
             153 | 
                 `----

            Qasm.Lowerer.GateCalledLikeFunc

              x gate called like function: gate(0, 1)
                 ,-[Test.qasm:155:1]
             154 | // CannotCallNonFunction
             155 | x(2);
                 : ^^^^
             156 | 
                 `----
              help: ensure that qubit arguments are provided to the gate call

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 1 classical arguments, but 2 were provided
                 ,-[Test.qasm:158:1]
             157 | // InvalidNumberOfClassicalArgs in gate call
             158 | rx(2.0, 3.0) q;
                 : ^^^^^^^^^^^^^^^
             159 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfQubitArgs

              x gate expects 1 qubit arguments, but 2 were provided
                 ,-[Test.qasm:161:1]
             160 | // InvalidNumberOfQubitArgs
             161 | rx(2.0) q, q;
                 : ^^^^^^^^^^^^^
             162 | 
                 `----

            Qasm.Lowerer.BroadcastCallQuantumArgsDisagreeInSize

              x first quantum register is of type qubit[1] but found an argument of type
              | qubit[2]
                 ,-[Test.qasm:164:18]
             163 | // BroadcastCallQuantumArgsDisagreeInSize
             164 | ryy(2.0) qreg_1, qreg_2;
                 :                  ^^^^^^
             165 | 
                 `----

            Qasm.Lowerer.ExprMustFitInU32

              x ctrl modifier argument must fit in a u32
                 ,-[Test.qasm:172:6]
             171 | // ExprMustFitInU32
             172 | ctrl(5000000000) @ x q;
                 :      ^^^^^^^^^^
             173 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:179:12]
             178 | // ArraySizeMustBeNonNegativeConstExpr
             179 | array[int, 2.0] non_int_array_size;
                 :            ^^^
             180 | 
                 `----

            Qasm.Lowerer.ExprMustBeNonNegativeInt

              x array size must be a non-negative integer
                 ,-[Test.qasm:182:12]
             181 | // ArraySizeMustBeNonNegativeConstExpr
             182 | array[int, -2] negative_array_size;
                 :            ^^
             183 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:185:12]
             184 | // DesignatorTooLarge
             185 | array[int, 5000000000] arr_size_too_large;
                 :            ^^^^^^^^^^
             186 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:188:5]
             187 | // TypeWidthMustBePositiveIntConstExpr
             188 | int[2.0] non_int_width;
                 :     ^^^
             189 | 
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:191:5]
             190 | // TypeWidthMustBePositiveIntConstExpr
             191 | int[0] zero_width;
                 :     ^
             192 | int[-2] negative_width;
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:192:5]
             191 | int[0] zero_width;
             192 | int[-2] negative_width;
                 :     ^^
             193 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:195:5]
             194 | // DesignatorTooLarge
             195 | int[5000000000] width_too_large;
                 :     ^^^^^^^^^^
             196 | 
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x float max width is 64 but 65 was provided
                 ,-[Test.qasm:198:1]
             197 | // TypeMaxWidthExceeded
             198 | float[65] float_width_too_large;
                 : ^^^^^^^^^
             199 | angle[65] angle_width_too_large;
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x angle max width is 64 but 65 was provided
                 ,-[Test.qasm:199:1]
             198 | float[65] float_width_too_large;
             199 | angle[65] angle_width_too_large;
                 : ^^^^^^^^^
             200 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type int
                 ,-[Test.qasm:202:1]
             201 | // Invalid literal cast in cast_expr_with_target_type_or_default(...)
             202 | int invalid_lit_cast = 2.0;
                 : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
             203 | 
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:211:5]
             210 | // QuantumTypesInBinaryExpression
             211 | 1 + q;
                 :     ^
             212 | q + 1;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:211:5]
             210 | // QuantumTypesInBinaryExpression
             211 | 1 + q;
                 :     ^
             212 | q + 1;
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:212:1]
             211 | 1 + q;
             212 | q + 1;
                 : ^
             213 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:212:1]
             211 | 1 + q;
             212 | q + 1;
                 : ^
             213 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type float
                 ,-[Test.qasm:216:1]
             215 | angle uncastable_to_int = 2.0;
             216 | uncastable_to_int + 3;
                 : ^^^^^^^^^^^^^^^^^
             217 | 3 + uncastable_to_int;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type const float
                 ,-[Test.qasm:217:5]
             216 | uncastable_to_int + 3;
             217 | 3 + uncastable_to_int;
                 :     ^^^^^^^^^^^^^^^^^
             218 | 
                 `----

            Qasm.Lowerer.OperatorNotAllowedForComplexValues

              x the operator OrB is not allowed for complex values
                 ,-[Test.qasm:220:1]
             219 | // OperatorNotAllowedForComplexValues
             220 | (2 + 1im) | 3im;
                 : ^^^^^^^^^^^^^^^
             221 | 
                 `----

            Qasm.Lowerer.IndexSetOnlyAllowedInAliasStmt

              x index sets are only allowed in alias statements
                 ,-[Test.qasm:223:8]
             222 | // IndexSetOnlyAllowedInAliasStmt
             223 | qreg_2[{0, 1}];
                 :        ^^^^^^
             224 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type const angle to type const int
                 ,-[Test.qasm:227:13]
             226 | array[int, 5] range_error;
             227 | range_error[const_uncastable_to_int:2.2];
                 :             ^^^^^^^^^^^^^^^^^^^^^^^
             228 | 
                 `----

            Qasm.Lowerer.ZeroStepInRange

              x range step cannot be zero
                 ,-[Test.qasm:230:13]
             229 | // ZeroStepInRange
             230 | range_error[1:0:3];
                 :             ^^^^^
             231 | 
                 `----

            Qasm.Lowerer.ZeroSizeArrayAccess

              x zero size array access is not allowed
                 ,-[Test.qasm:234:1]
             233 | array[int, 2, 0, 3] zero_size_array;
             234 | zero_size_array[1];
                 : ^^^^^^^^^^^^^^^^^^
             235 | 
                 `----
              help: array size must be a positive integer const expression

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type bit
                 ,-[Test.qasm:238:15]
             237 | bit non_indexable;
             238 | non_indexable[1];
                 :               ^
             239 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type qubit
                 ,-[Test.qasm:241:11]
             240 | // TooManyIndices
             241 | qreg_1[1, 2];
                 :           ^
             242 | 
                 `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: missing_symbol
                 ,-[Test.qasm:244:1]
             243 | // Missing symbol in lower_indexed_ident_expr(...)
             244 | missing_symbol[2];
                 : ^^^^^^^^^^^^^^
             245 | 
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type unknown
                 ,-[Test.qasm:244:16]
             243 | // Missing symbol in lower_indexed_ident_expr(...)
             244 | missing_symbol[2];
                 :                ^
             245 | 
                 `----

            Qasm.Lowerer.EmptyIndexOperator

              x index operator must contain at least one index
                 ,-[Test.qasm:248:13]
             247 | bit[4] empty_index;
             248 | empty_index[];
                 :             ^
             249 | 
                 `----

            Qasm.Lowerer.CannotCallNonFunction

              x cannot call an expression that is not a function
                 ,-[Test.qasm:251:1]
             250 | // CannotCallNonFunction
             251 | empty_index();
                 : ^^^^^^^^^^^^^
             252 | 
                 `----

            Qasm.Lowerer.FuncCalledLikeGate

              x function called like gate: def (qubit) -> void
                 ,-[Test.qasm:254:5]
             253 | // FuncCalledLikeGate
             254 | def func_called_like_gate(qubit q) {}
                 :     ^^^^^^^^^^^^^^^^^^^^^
             255 | func_called_like_gate q;
                 `----
              help: function parameters must be in parentheses

            Qasm.Lowerer.GateCallMissingParams

              x gate call missing parameters: gate(0, 1)
                 ,-[Test.qasm:258:1]
             257 | // GateCallMissingParams
             258 | h;
                 : ^
             259 | 
                 `----
              help: ensure that any classical and quantum arguments are provided to the
                    gate call

            Qasm.Lowerer.FuncMissingParams

              x function call missing parameters: def (qubit) -> void
                 ,-[Test.qasm:262:1]
             261 | // FuncMissingParams
             262 | func_called_like_gate;
                 : ^^^^^^^^^^^^^^^^^^^^^
                 `----
              help: a function call must use parentheses, with any parameters inside
                    those parentheses.
        "#]],
    );
}
