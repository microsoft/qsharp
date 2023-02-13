// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) const ADJ: &str = "Adj";

pub(super) const ADJOINT: &str = "adjoint";

pub(super) const ADJOINT_UPPER: &str = "Adjoint";

pub(super) const AND: &str = "and";

pub(super) const APPLY: &str = "apply";

pub(super) const AS: &str = "as";

pub(super) const AUTO: &str = "auto";

pub(super) const BIG_INT: &str = "BigInt";

pub(super) const BODY: &str = "body";

pub(super) const BOOL: &str = "Bool";

pub(super) const BORROW: &str = "borrow";

pub(super) const BORROWING: &str = "borrowing";

pub(super) const CONTROLLED: &str = "controlled";

pub(super) const CONTROLLED_UPPER: &str = "Controlled";

pub(super) const CTL: &str = "Ctl";

pub(super) const DISTRIBUTE: &str = "distribute";

pub(super) const DOUBLE: &str = "Double";

pub(super) const ELIF: &str = "elif";

pub(super) const ELSE: &str = "else";

pub(super) const FAIL: &str = "fail";

pub(super) const FALSE: &str = "false";

pub(super) const FIXUP: &str = "fixup";

pub(super) const FOR: &str = "for";

pub(super) const FUNCTION: &str = "function";

pub(super) const IF: &str = "if";

pub(super) const IN: &str = "in";

pub(super) const INT: &str = "Int";

pub(super) const INTERNAL: &str = "internal";

pub(super) const INTRINSIC: &str = "intrinsic";

pub(super) const INVERT: &str = "invert";

pub(super) const IS: &str = "is";

pub(super) const LET: &str = "let";

pub(super) const MUTABLE: &str = "mutable";

pub(super) const NAMESPACE: &str = "namespace";

pub(super) const NEWTYPE: &str = "newtype";

pub(super) const NOT: &str = "not";

pub(super) const ONE: &str = "One";

pub(super) const OPEN: &str = "open";

pub(super) const OPERATION: &str = "operation";

pub(super) const OR: &str = "or";

pub(super) const PAULI: &str = "Pauli";

pub(super) const PAULI_I: &str = "PauliI";

pub(super) const PAULI_X: &str = "PauliX";

pub(super) const PAULI_Z: &str = "PauliZ";

pub(super) const QUBIT: &str = "Qubit";

pub(super) const RANGE: &str = "Range";

pub(super) const REPEAT: &str = "repeat";

pub(super) const RESULT: &str = "Result";

pub(super) const RETURN: &str = "return";

pub(super) const SELF: &str = "self";

pub(super) const SET: &str = "set";

pub(super) const STRING: &str = "String";

pub(super) const TRUE: &str = "true";

pub(super) const UNIT: &str = "Unit";

pub(super) const UNTIL: &str = "until";

pub(super) const USE: &str = "use";

pub(super) const USING: &str = "using";

pub(super) const WHILE: &str = "while";

pub(super) const WITHIN: &str = "within";

pub(super) const ZERO: &str = "Zero";

pub(super) fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        ADJ | ADJOINT
            | ADJOINT_UPPER
            | AND
            | APPLY
            | AS
            | AUTO
            | BIG_INT
            | BODY
            | BOOL
            | BORROW
            | BORROWING
            | CONTROLLED
            | CONTROLLED_UPPER
            | CTL
            | DISTRIBUTE
            | DOUBLE
            | ELIF
            | ELSE
            | FAIL
            | FALSE
            | FIXUP
            | FOR
            | FUNCTION
            | IF
            | IN
            | INT
            | INTERNAL
            | INTRINSIC
            | INVERT
            | IS
            | LET
            | MUTABLE
            | NAMESPACE
            | NEWTYPE
            | NOT
            | ONE
            | OPEN
            | OPERATION
            | OR
            | PAULI
            | PAULI_I
            | PAULI_X
            | PAULI_Z
            | QUBIT
            | RANGE
            | REPEAT
            | RESULT
            | RETURN
            | SELF
            | SET
            | STRING
            | TRUE
            | UNIT
            | UNTIL
            | USE
            | USING
            | WHILE
            | WITHIN
            | ZERO
    )
}
