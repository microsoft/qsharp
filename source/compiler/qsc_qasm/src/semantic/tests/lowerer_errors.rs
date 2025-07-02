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

            Qasm.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: pragma stmt
               ,-[Test.qasm:7:1]
             6 | // Unimplemented pragma
             7 | pragma my_pragma;
               : ^^^^^^^^^^^^^^^^^
             8 | 
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

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const int to type const bit
                ,-[Test.qasm:51:1]
             50 | // CannotUpdateConstVariable in indexed assign_op
             51 | const_bitarray[1] += 7;
                : ^^^^^^^^^^^^^^^^^^^^^^^
             52 | 
                `----

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

              x duration type values are not supported
                ,-[Test.qasm:80:1]
             79 | // NotSupported duration type
             80 | duration duration_val;
                : ^^^^^^^^
             81 | 
                `----

            Qasm.Lowerer.NotSupported

              x stretch type values are not supported
                ,-[Test.qasm:83:1]
             82 | // NotSupported stretch type
             83 | stretch stretch_val;
                : ^^^^^^^
             84 | 
                `----

            Qasm.Lowerer.NotSupported

              x stretch default values are not supported
                ,-[Test.qasm:83:1]
             82 | // NotSupported stretch type
             83 | stretch stretch_val;
                : ^^^^^^^^^^^^^^^^^^^^
             84 | 
                `----

            Qasm.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
                ,-[Test.qasm:88:5]
             87 |     // ClassicalStmtInBox
             88 |     2;
                :     ^^
             89 | }
                `----

            Qasm.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: box stmt
                ,-[Test.qasm:86:1]
             85 |     // Unimplemented box
             86 | ,-> box {
             87 | |       // ClassicalStmtInBox
             88 | |       2;
             89 | `-> }
             90 |     
                `----

            Qasm.Lowerer.InvalidScope

              x break can only appear in loop scopes
                ,-[Test.qasm:92:1]
             91 | // InvalidScope break outside loop
             92 | break;
                : ^^^^^^
             93 | 
                `----

            Qasm.Lowerer.InvalidScope

              x continue can only appear in loop scopes
                ,-[Test.qasm:95:1]
             94 | // InvalidScope continue outside loop
             95 | continue;
                : ^^^^^^^^^
             96 | 
                `----

            Qasm.Lowerer.InvalidScope

              x return statements can only appear in subroutine scopes
                ,-[Test.qasm:98:1]
             97 | // InvalidScope return outside def
             98 | return;
                : ^^^^^^^
             99 | 
                `----

            Qasm.Lowerer.MissingTargetExpressionInReturnStmt

              x return statements on a non-void subroutine should have a target expression
                 ,-[Test.qasm:102:5]
             101 | def missing_target_in_return() -> int {
             102 |     return;
                 :     ^^^^^^^
             103 | }
                 `----

            Qasm.Lowerer.ReturningExpressionFromVoidSubroutine

              x cannot return an expression from a void subroutine
                 ,-[Test.qasm:107:12]
             106 | def returning_from_void_subroutine() {
             107 |     return 2;
                 :            ^
             108 | }
                 `----

            Qasm.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: def cal stmt
                 ,-[Test.qasm:111:1]
             110 | // Unimplemented defcal
             111 | defcal {}
                 : ^^^^^^^^^
             112 | 
                 `----

            Qasm.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: calibration
              | grammar stmt
                 ,-[Test.qasm:114:1]
             113 | // Unimplemented defcalgrammar
             114 | defcalgrammar "my_grammar";
                 : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
             115 | 
                 `----

            Qasm.Lowerer.ExprMustBeConst

              x const decl init expr must be a const expression
                 ,-[Test.qasm:118:23]
             117 | int non_const_val = 2;
             118 | const int const_val = non_const_val;
                 :                       ^^^^^^^^^^^^^
             119 | 
                 `----

              x array declarations are only allowed in global scope
                 ,-[Test.qasm:122:5]
             121 | {
             122 |     array[int, 1, 2] arr;
                 :     ^^^^^^^^^^^^^^^^^^^^^
             123 | }
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x def declarations must be done in global scope
                 ,-[Test.qasm:127:5]
             126 | {
             127 |     def f() {}
                 :     ^^^^^^^^^^
             128 | }
                 `----

            Qasm.Lowerer.GateDeclarationInNonGlobalScope

              x gate declarations must be done in global scope
                 ,-[Test.qasm:132:5]
             131 | {
             132 |     gate g q {}
                 :     ^^^^^^^^^^^
             133 | }
                 `----

            Qasm.Lowerer.QubitDeclarationInNonGlobalScope

              x qubit declarations must be done in global scope
                 ,-[Test.qasm:137:5]
             136 | {
             137 |     qubit non_global_qubit;
                 :     ^^^^^^^^^^^^^^^^^^^^^^^
             138 | }
                 `----

            Qasm.Lowerer.NonVoidDefShouldAlwaysReturn

              x non-void def should always return
                 ,-[Test.qasm:141:37]
             140 | // NonVoidDefShouldAlwaysReturn
             141 | def non_void_def_should_return() -> int {
                 :                                     ^^^
             142 | 
                 `----

            Qasm.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: delay stmt
                 ,-[Test.qasm:146:1]
             145 | // Unimplemented delay
             146 | delay [2ns] q1;
                 : ^^^^^^^^^^^^^^^
             147 | 
                 `----

            Qasm.Lowerer.DefDeclarationInNonGlobalScope

              x extern declarations must be done in global scope
                 ,-[Test.qasm:151:5]
             150 | {
             151 |     extern f(int);
                 :     ^^^^^^^^^^^^^^
             152 | }
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 2 classical arguments, but 1 were provided
                 ,-[Test.qasm:156:1]
             155 | def invalid_arity_call(int a, int b) {}
             156 | invalid_arity_call(2);
                 : ^^^^^^^^^^^^^^^^^^^^^
             157 | 
                 `----

            Qasm.Lowerer.CannotCallNonFunction

              x cannot call an expression that is not a function
                 ,-[Test.qasm:159:1]
             158 | // CannotCallNonFunction
             159 | x(2);
                 : ^^^^
             160 | 
                 `----

            Qasm.Lowerer.NotSupported

              x gate call duration are not supported
                 ,-[Test.qasm:162:3]
             161 | // NotSupported gate call duration
             162 | x[2ns] q;
                 :   ^^^
             163 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfClassicalArgs

              x gate expects 1 classical arguments, but 2 were provided
                 ,-[Test.qasm:165:1]
             164 | // InvalidNumberOfClassicalArgs in gate call
             165 | rx(2.0, 3.0) q;
                 : ^^^^^^^^^^^^^^^
             166 | 
                 `----

            Qasm.Lowerer.InvalidNumberOfQubitArgs

              x gate expects 1 qubit arguments, but 2 were provided
                 ,-[Test.qasm:168:1]
             167 | // InvalidNumberOfQubitArgs
             168 | rx(2.0) q, q;
                 : ^^^^^^^^^^^^^
             169 | 
                 `----

            Qasm.Lowerer.BroadcastCallQuantumArgsDisagreeInSize

              x first quantum register is of type qubit[1] but found an argument of type
              | qubit[2]
                 ,-[Test.qasm:171:18]
             170 | // BroadcastCallQuantumArgsDisagreeInSize
             171 | ryy(2.0) qreg_1, qreg_2;
                 :                  ^^^^^^
             172 | 
                 `----

            Qasm.Lowerer.ExprMustFitInU32

              x ctrl modifier argument must fit in a u32
                 ,-[Test.qasm:179:6]
             178 | // ExprMustFitInU32
             179 | ctrl(5000000000) @ x q;
                 :      ^^^^^^^^^^
             180 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:186:12]
             185 | // ArraySizeMustBeNonNegativeConstExpr
             186 | array[int, 2.0] non_int_array_size;
                 :            ^^^
             187 | 
                 `----

            Qasm.Lowerer.ExprMustBeNonNegativeInt

              x array size must be a non-negative integer
                 ,-[Test.qasm:189:12]
             188 | // ArraySizeMustBeNonNegativeConstExpr
             189 | array[int, -2] negative_array_size;
                 :            ^^
             190 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:192:12]
             191 | // DesignatorTooLarge
             192 | array[int, 5000000000] arr_size_too_large;
                 :            ^^^^^^^^^^
             193 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type const uint
                 ,-[Test.qasm:195:5]
             194 | // TypeWidthMustBePositiveIntConstExpr
             195 | int[2.0] non_int_width;
                 :     ^^^
             196 | 
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:198:5]
             197 | // TypeWidthMustBePositiveIntConstExpr
             198 | int[0] zero_width;
                 :     ^
             199 | int[-2] negative_width;
                 `----

            Qasm.Lowerer.ExprMustBePositiveInt

              x type width must be a positive integer
                 ,-[Test.qasm:199:5]
             198 | int[0] zero_width;
             199 | int[-2] negative_width;
                 :     ^^
             200 | 
                 `----

            Qasm.Lowerer.DesignatorTooLarge

              x designator is too large
                 ,-[Test.qasm:202:5]
             201 | // DesignatorTooLarge
             202 | int[5000000000] width_too_large;
                 :     ^^^^^^^^^^
             203 | 
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x float max width is 64 but 65 was provided
                 ,-[Test.qasm:205:1]
             204 | // TypeMaxWidthExceeded
             205 | float[65] float_width_too_large;
                 : ^^^^^^^^^
             206 | angle[65] angle_width_too_large;
                 `----

            Qasm.Lowerer.TypeMaxWidthExceeded

              x angle max width is 64 but 65 was provided
                 ,-[Test.qasm:206:1]
             205 | float[65] float_width_too_large;
             206 | angle[65] angle_width_too_large;
                 : ^^^^^^^^^
             207 | 
                 `----

            Qasm.Lowerer.CannotCastLiteral

              x cannot cast literal expression of type const float to type int
                 ,-[Test.qasm:209:1]
             208 | // Invalid literal cast in cast_expr_with_target_type_or_default(...)
             209 | int invalid_lit_cast = 2.0;
                 : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
             210 | 
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:218:5]
             217 | // QuantumTypesInBinaryExpression
             218 | 1 + q;
                 :     ^
             219 | q + 1;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:218:5]
             217 | // QuantumTypesInBinaryExpression
             218 | 1 + q;
                 :     ^
             219 | q + 1;
                 `----

            Qasm.Lowerer.QuantumTypesInBinaryExpression

              x quantum typed values cannot be used in binary expressions
                 ,-[Test.qasm:219:1]
             218 | 1 + q;
             219 | q + 1;
                 : ^
             220 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type qubit to type const float
                 ,-[Test.qasm:219:1]
             218 | 1 + q;
             219 | q + 1;
                 : ^
             220 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type float
                 ,-[Test.qasm:223:1]
             222 | angle uncastable_to_int = 2.0;
             223 | uncastable_to_int + 3;
                 : ^^^^^^^^^^^^^^^^^
             224 | 3 + uncastable_to_int;
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type angle to type const float
                 ,-[Test.qasm:224:5]
             223 | uncastable_to_int + 3;
             224 | 3 + uncastable_to_int;
                 :     ^^^^^^^^^^^^^^^^^
             225 | 
                 `----

            Qasm.Lowerer.OperatorNotAllowedForComplexValues

              x the operator OrB is not allowed for complex values
                 ,-[Test.qasm:227:1]
             226 | // OperatorNotAllowedForComplexValues
             227 | (2 + 1im) | 3im;
                 : ^^^^^^^^^^^^^^^
             228 | 
                 `----

            Qasm.Lowerer.IndexSetOnlyAllowedInAliasStmt

              x index sets are only allowed in alias statements
                 ,-[Test.qasm:230:8]
             229 | // IndexSetOnlyAllowedInAliasStmt
             230 | qreg_2[{0, 1}];
                 :        ^^^^^^
             231 | 
                 `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type const angle to type const int
                 ,-[Test.qasm:234:13]
             233 | array[int, 5] range_error;
             234 | range_error[const_uncastable_to_int:2.2];
                 :             ^^^^^^^^^^^^^^^^^^^^^^^
             235 | 
                 `----

            Qasm.Lowerer.ZeroStepInRange

              x range step cannot be zero
                 ,-[Test.qasm:237:13]
             236 | // ZeroStepInRange
             237 | range_error[1:0:3];
                 :             ^^^^^
             238 | 
                 `----

            Qasm.Lowerer.ZeroSizeArrayAccess

              x zero size array access is not allowed
                 ,-[Test.qasm:241:1]
             240 | array[int, 2, 0, 3] zero_size_array;
             241 | zero_size_array[1];
                 : ^^^^^^^^^^^^^^^^^^
             242 | 
                 `----
              help: array size must be a positive integer const expression

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type bit
                 ,-[Test.qasm:245:1]
             244 | bit non_indexable;
             245 | non_indexable[1];
                 : ^^^^^^^^^^^^^^^^
             246 | 
                 `----

            Qasm.Lowerer.TooManyIndices

              x too many indices specified
                 ,-[Test.qasm:248:1]
             247 | // TooManyIndices
             248 | qreg_1[1, 2];
                 : ^^^^^^^^^^^^
             249 | 
                 `----

            Qasm.Lowerer.UndefinedSymbol

              x undefined symbol: missing_symbol
                 ,-[Test.qasm:252:1]
             251 | // Missing symbol in lower_indexed_ident_expr(...)
             252 | missing_symbol[2];
                 : ^^^^^^^^^^^^^^
                 `----

            Qasm.Lowerer.CannotIndexType

              x cannot index variables of type unknown
                 ,-[Test.qasm:252:1]
             251 | // Missing symbol in lower_indexed_ident_expr(...)
             252 | missing_symbol[2];
                 : ^^^^^^^^^^^^^^^^^
                 `----
        "#]],
    );
}
