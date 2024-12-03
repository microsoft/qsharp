// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};

/// Each file in the samples/language folder is compiled and run as two tests and should
/// have matching expect strings in this file. If new samples are added, this file will
/// fail to compile until the new expect strings are added.
pub const ARITHMETICOPERATORS_EXPECT: Expect = expect!["()"];
pub const ARITHMETICOPERATORS_EXPECT_DEBUG: Expect = expect!["()"];
pub const ARRAY_EXPECT: Expect = expect![[r#"
    Integer Array: [1, 2, 3, 4] of length 4
    String Array: [a, string, array]
    Repeated Array: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    Repeated Array: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    Sliced array: [2, 4]
    Sliced array: [3, 2, 1]
    Sliced array: [1, 2, 3, 4]
    [1, 2, 3, 4]"#]];
pub const ARRAY_EXPECT_DEBUG: Expect = expect![[r#"
    Integer Array: [1, 2, 3, 4] of length 4
    String Array: [a, string, array]
    Repeated Array: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    Repeated Array: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    Sliced array: [2, 4]
    Sliced array: [3, 2, 1]
    Sliced array: [1, 2, 3, 4]
    [1, 2, 3, 4]"#]];
pub const BIGINT_EXPECT: Expect = expect![[r#"
    Hexadecimal BigInt: 66
    Octal BigInt: 34
    Decimal BigInt: 42
    Binary BigInt: 42
    Addition result: 43
    Modulo result: 1
    Exponentiation result: 1
    1"#]];
pub const BIGINT_EXPECT_DEBUG: Expect = expect![[r#"
    Hexadecimal BigInt: 66
    Octal BigInt: 34
    Decimal BigInt: 42
    Binary BigInt: 42
    Addition result: 43
    Modulo result: 1
    Exponentiation result: 1
    1"#]];
pub const BITWISEOPERATORS_EXPECT: Expect = expect![[r#"
    Bitwise NOT: -6
    Bitwise NOT: 4
    Bitwise AND: 4
    Bitwise AND: 2
    Bitwise OR: 7
    Bitwise OR: -1
    Bitwise XOR: 3
    Bitwise XOR: -3
    Right Bit-shift: 1
    Right Bit-shift: -2
    Right Bit-shift: 20
    Left Bit-shift: 20
    Left Bit-shift: -20
    Left Bit-shift: 1
    ()"#]];
pub const BITWISEOPERATORS_EXPECT_DEBUG: Expect = expect![[r#"
    Bitwise NOT: -6
    Bitwise NOT: 4
    Bitwise AND: 4
    Bitwise AND: 2
    Bitwise OR: 7
    Bitwise OR: -1
    Bitwise XOR: 3
    Bitwise XOR: -3
    Right Bit-shift: 1
    Right Bit-shift: -2
    Right Bit-shift: 20
    Left Bit-shift: 20
    Left Bit-shift: -20
    Left Bit-shift: 1
    ()"#]];
pub const BOOL_EXPECT: Expect = expect![[r#"
    AND operation: true
    OR operation: true
    Equality comparison: false
    2 equals 2
    true"#]];
pub const BOOL_EXPECT_DEBUG: Expect = expect![[r#"
    AND operation: true
    OR operation: true
    Equality comparison: false
    2 equals 2
    true"#]];
pub const COMMENTS_EXPECT: Expect = expect!["[]"];
pub const COMMENTS_EXPECT_DEBUG: Expect = expect!["[]"];
pub const COMPARISONOPERATORS_EXPECT: Expect = expect![[r#"
    Equality comparison: true
    Equality comparison: false
    Inequality comparison: false
    Inequality comparison: true
    Less than comparison: false
    Less than comparison: true
    Less than comparison: false
    Less than or equal comparison: true
    Less than or equal comparison: true
    Less than or equal comparison: false
    Greater than comparison: false
    Greater than comparison: false
    Greater than comparison: true
    Greater than or equal comparison: true
    Greater than or equal comparison: false
    Greater than or equal comparison: true
    ()"#]];
pub const COMPARISONOPERATORS_EXPECT_DEBUG: Expect = expect![[r#"
    Equality comparison: true
    Equality comparison: false
    Inequality comparison: false
    Inequality comparison: true
    Less than comparison: false
    Less than comparison: true
    Less than comparison: false
    Less than or equal comparison: true
    Less than or equal comparison: true
    Less than or equal comparison: false
    Greater than comparison: false
    Greater than comparison: false
    Greater than comparison: true
    Greater than or equal comparison: true
    Greater than or equal comparison: false
    Greater than or equal comparison: true
    ()"#]];
pub const CONDITIONALBRANCHING_EXPECT: Expect = expect![[r#"
    Buzz
    It is livable
    Absolute value of -40 is 40
    ()"#]];
pub const CONDITIONALBRANCHING_EXPECT_DEBUG: Expect = expect![[r#"
    Buzz
    It is livable
    Absolute value of -40 is 40
    ()"#]];
pub const COPYANDUPDATEOPERATOR_EXPECT: Expect = expect![[r#"
    Updated array: [10, 11, 100, 13]
    Updated array: [10, 100, 12, 200]
    ()"#]];
pub const COPYANDUPDATEOPERATOR_EXPECT_DEBUG: Expect = expect![[r#"
    Updated array: [10, 11, 100, 13]
    Updated array: [10, 100, 12, 200]
    ()"#]];
pub const CUSTOMMEASUREMENTS_EXPECT: Expect = expect!["Zero"];
pub const CUSTOMMEASUREMENTS_EXPECT_DEBUG: Expect = expect!["Zero"];
pub const DATATYPES_EXPECT: Expect = expect![[r#"
    Binary BigInt: 42
    Octal BigInt: 42
    Decimal BigInt: 42
    Hexadecimal BigInt: 42
    Complex: (real: 42.0, imaginary: 0.0)
    ()"#]];
pub const DATATYPES_EXPECT_DEBUG: Expect = expect![[r#"
    Binary BigInt: 42
    Octal BigInt: 42
    Decimal BigInt: 42
    Hexadecimal BigInt: 42
    Complex: (real: 42.0, imaginary: 0.0)
    ()"#]];
pub const DIAGNOSTICS_EXPECT: Expect = expect![[r#"
    Program is starting.
    STATE:
    |00⟩: 0.7071+0.0000𝑖
    |10⟩: 0.7071+0.0000𝑖
    ()"#]];
pub const DIAGNOSTICS_EXPECT_DEBUG: Expect = expect![[r#"
    Program is starting.
    STATE:
    |00⟩: 0.7071+0.0000𝑖
    |10⟩: 0.7071+0.0000𝑖
    ()"#]];
pub const DOUBLE_EXPECT: Expect = expect!["0.1973269804"];
pub const DOUBLE_EXPECT_DEBUG: Expect = expect!["0.1973269804"];
pub const ENTRYPOINT_EXPECT: Expect = expect!["[]"];
pub const ENTRYPOINT_EXPECT_DEBUG: Expect = expect!["[]"];
pub const FAILSTATEMENT_EXPECT: Expect = expect!["()"];
pub const FAILSTATEMENT_EXPECT_DEBUG: Expect = expect!["()"];
pub const FORLOOPS_EXPECT: Expect = expect!["()"];
pub const FORLOOPS_EXPECT_DEBUG: Expect = expect!["()"];
pub const FUNCTIONS_EXPECT: Expect = expect!["()"];
pub const FUNCTIONS_EXPECT_DEBUG: Expect = expect!["()"];
pub const GETTINGSTARTED_EXPECT: Expect = expect!["()"];
pub const GETTINGSTARTED_EXPECT_DEBUG: Expect = expect!["()"];
pub const INT_EXPECT: Expect = expect![[r#"
    Hexadecimal: 66
    Octal: 34
    Decimal: 42
    Binary: 42
    After addition: 43
    After modulo: 1
    After exponentiation: 1
    1"#]];
pub const INT_EXPECT_DEBUG: Expect = expect![[r#"
    Hexadecimal: 66
    Octal: 34
    Decimal: 42
    Binary: 42
    After addition: 43
    After modulo: 1
    After exponentiation: 1
    1"#]];
pub const LAMBDAEXPRESSION_EXPECT: Expect = expect![[r#"
    Lambda add function result: 5
    Sum of array using Fold: 15
    Array after incrementing each element using Map: [2, 3, 4, 5, 6]
    ()"#]];
pub const LAMBDAEXPRESSION_EXPECT_DEBUG: Expect = expect![[r#"
    Lambda add function result: 5
    Sum of array using Fold: 15
    Array after incrementing each element using Map: [2, 3, 4, 5, 6]
    ()"#]];
pub const LOGICALOPERATORS_EXPECT: Expect = expect!["()"];
pub const LOGICALOPERATORS_EXPECT_DEBUG: Expect = expect!["()"];
pub const NAMESPACES_EXPECT: Expect = expect![[r#"
    STATE:
    No qubits allocated
    []"#]];
pub const NAMESPACES_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    No qubits allocated
    []"#]];
pub const OPERATIONS_EXPECT: Expect = expect![[r#"
    Measurement result: Zero
    Zero"#]];
pub const OPERATIONS_EXPECT_DEBUG: Expect = expect![[r#"
    Measurement result: Zero
    Zero"#]];
pub const PARTIALAPPLICATION_EXPECT: Expect = expect![[r#"
    five = incrementByOne(4) => 5
    Incremented array: [2, 3, 4, 5, 6]
    ()"#]];
pub const PARTIALAPPLICATION_EXPECT_DEBUG: Expect = expect![[r#"
    five = incrementByOne(4) => 5
    Incremented array: [2, 3, 4, 5, 6]
    ()"#]];
pub const PAULI_EXPECT: Expect = expect![[r#"
    Pauli dimension: PauliX
    Measurement result: Zero
    Zero"#]];
pub const PAULI_EXPECT_DEBUG: Expect = expect![[r#"
    Pauli dimension: PauliX
    Measurement result: Zero
    Zero"#]];
pub const QUANTUMMEMORY_EXPECT: Expect = expect!["()"];
pub const QUANTUMMEMORY_EXPECT_DEBUG: Expect = expect!["()"];
pub const QUBIT_EXPECT: Expect = expect![[r#"
    STATE:
    |1000⟩: 0.0000+0.5000𝑖
    |1010⟩: 0.0000+0.5000𝑖
    |1100⟩: 0.0000+0.5000𝑖
    |1110⟩: 0.0000+0.5000𝑖
    ()"#]];
pub const QUBIT_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    |1000⟩: 0.0000+0.5000𝑖
    |1010⟩: 0.0000+0.5000𝑖
    |1100⟩: 0.0000+0.5000𝑖
    |1110⟩: 0.0000+0.5000𝑖
    ()"#]];
pub const RANGE_EXPECT: Expect = expect![[r#"
    Range: 1..3
    Range: 2..2..5
    Range: 2..2..6
    Range: 6..-2..2
    Range: 2..-2..2
    Range: 2..1
    Array: [0, 1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
    Array slice [0..2..10]: [0, 4, 16, 36, 64, 100]
    Array slice [...4]: [0, 1, 4, 9, 16]
    Array slice [5...]: [25, 36, 49, 64, 81, 100]
    Array slice [2..3...]: [4, 25, 64]
    Array slice [...3..7]: [0, 9, 36]
    Array slice [...]: [0, 1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
    Array slice [...-3...]: [100, 49, 16, 1]
    2..1"#]];
pub const RANGE_EXPECT_DEBUG: Expect = expect![[r#"
    Range: 1..3
    Range: 2..2..5
    Range: 2..2..6
    Range: 6..-2..2
    Range: 2..-2..2
    Range: 2..1
    Array: [0, 1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
    Array slice [0..2..10]: [0, 4, 16, 36, 64, 100]
    Array slice [...4]: [0, 1, 4, 9, 16]
    Array slice [5...]: [25, 36, 49, 64, 81, 100]
    Array slice [2..3...]: [4, 25, 64]
    Array slice [...3..7]: [0, 9, 36]
    Array slice [...]: [0, 1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
    Array slice [...-3...]: [100, 49, 16, 1]
    2..1"#]];
pub const REPEATUNTILLOOPS_EXPECT: Expect = expect!["()"];
pub const REPEATUNTILLOOPS_EXPECT_DEBUG: Expect = expect!["()"];
pub const RESULT_EXPECT: Expect = expect![[r#"
    Measurement: Zero
    Zero"#]];
pub const RESULT_EXPECT_DEBUG: Expect = expect![[r#"
    Measurement: Zero
    Zero"#]];
pub const RETURNSTATEMENT_EXPECT: Expect = expect!["()"];
pub const RETURNSTATEMENT_EXPECT_DEBUG: Expect = expect!["()"];
pub const SPECIALIZATIONS_EXPECT: Expect = expect!["()"];
pub const SPECIALIZATIONS_EXPECT_DEBUG: Expect = expect!["()"];
pub const STRING_EXPECT: Expect = expect![[r#"
    FooBar
    interpolated: FooBar
    interpolated: FooBar"#]];
pub const STRING_EXPECT_DEBUG: Expect = expect![[r#"
    FooBar
    interpolated: FooBar
    interpolated: FooBar"#]];
pub const TERNARY_EXPECT: Expect = expect![[r#"
    Absolute value: 40
    ()"#]];
pub const TERNARY_EXPECT_DEBUG: Expect = expect![[r#"
    Absolute value: 40
    ()"#]];
pub const TUPLE_EXPECT: Expect = expect![[r#"
    Tuple: (Id, 0, 1.0)
    Tuple: (PauliX, (3, 1))
    (0, Foo)"#]];
pub const TUPLE_EXPECT_DEBUG: Expect = expect![[r#"
    Tuple: (Id, 0, 1.0)
    Tuple: (PauliX, (3, 1))
    (0, Foo)"#]];
pub const TYPEDECLARATIONS_EXPECT: Expect = expect!["()"];
pub const TYPEDECLARATIONS_EXPECT_DEBUG: Expect = expect!["()"];
pub const UNIT_EXPECT: Expect = expect!["()"];
pub const UNIT_EXPECT_DEBUG: Expect = expect!["()"];
pub const VARIABLES_EXPECT: Expect = expect![[r#"
    Immutable Int: 42
    Mutable Int: 43
    Mutable Int after mutation: 42
    Shadowed Immutable Int: 0
    ()"#]];
pub const VARIABLES_EXPECT_DEBUG: Expect = expect![[r#"
    Immutable Int: 42
    Mutable Int: 43
    Mutable Int after mutation: 42
    Shadowed Immutable Int: 0
    ()"#]];
pub const WHILELOOPS_EXPECT: Expect = expect!["()"];
pub const WHILELOOPS_EXPECT_DEBUG: Expect = expect!["()"];
pub const WITHINAPPLY_EXPECT: Expect = expect!["()"];
pub const WITHINAPPLY_EXPECT_DEBUG: Expect = expect!["()"];
pub const CLASSCONSTRAINTS_EXPECT: Expect = expect![[r#"
    true
    false
    false
    false
    false
    true
    ()"#]];
pub const CLASSCONSTRAINTS_EXPECT_DEBUG: Expect = expect![[r#"
    true
    false
    false
    false
    false
    true
    ()"#]];
