// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{run_stdlib_test, run_stdlib_test_expression};
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Arrays namespace

#[test]
fn check_all() {
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.All(x -> x != 0, [1, 2, 3, 4, 5])
        }"#,
        &Value::Bool(true),
    );
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.All(x -> x != 0, [1, 2, 0, 4, 5])
        }"#,
        &Value::Bool(false),
    );
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.All(x -> x == One, [One, One, One])
        }"#,
        &Value::Bool(true),
    );
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.All(x -> x == One, [One, One, Zero])
        }"#,
        &Value::Bool(false),
    );
}

#[test]
fn check_any() {
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.Any(x -> x % 2 == 0, [1, 3, 6, 7, 9])
        }"#,
        &Value::Bool(true),
    );
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.Any(x -> x % 2 == 0, [1, 3, 5, 7, 9])
        }"#,
        &Value::Bool(false),
    );
}

#[test]
fn check_chunks() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Chunks(1, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10)].into()),
                Value::Array(vec![Value::Int(11)].into()),
                Value::Array(vec![Value::Int(12)].into()),
                Value::Array(vec![Value::Int(13)].into()),
                Value::Array(vec![Value::Int(14)].into()),
                Value::Array(vec![Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Chunks(2, [])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Chunks(2, [10])",
        &Value::Array(vec![Value::Array(vec![Value::Int(10)].into())].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Chunks(2, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10), Value::Int(11)].into()),
                Value::Array(vec![Value::Int(12), Value::Int(13)].into()),
                Value::Array(vec![Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Chunks(3, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10), Value::Int(11), Value::Int(12)].into()),
                Value::Array(vec![Value::Int(13), Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Chunks(4, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(
                    vec![
                        Value::Int(10),
                        Value::Int(11),
                        Value::Int(12),
                        Value::Int(13),
                    ]
                    .into(),
                ),
                Value::Array(vec![Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_column_at() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.ColumnAt(0, [[1, 2, 3], [4, 5, 6], [7, 8, 9]])",
        &Value::Array(vec![Value::Int(1), Value::Int(4), Value::Int(7)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.ColumnAt(2, [[true, true, true], [false, false, false]])",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.ColumnAt(1, [[One, One], [Zero, Zero], [Zero, One]])",
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(false),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_count() {
    run_stdlib_test(
        r#"operation Test() : Int {
            Microsoft.Quantum.Arrays.Count(x -> x % 2 != 0, [1, 3, 6, 7, 9])
        }"#,
        &Value::Int(4),
    );
    run_stdlib_test(
        r#"operation Test() : Int {
            Microsoft.Quantum.Arrays.Count(x -> x % 2 == 0, [1, 3, 6, 7, 9])
        }"#,
        &Value::Int(1),
    );
}

#[test]
fn check_diagnonal() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1]])",
        &Value::Array(vec![Value::Int(1)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2, 3], [4, 5, 6], [7, 8, 9]])",
        &Value::Array(vec![Value::Int(1), Value::Int(5), Value::Int(9)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2, 3], [4, 5, 6]])",
        &Value::Array(vec![Value::Int(1), Value::Int(5)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2], [3, 4], [5, 6]])",
        &Value::Array(vec![Value::Int(1), Value::Int(4)].into()),
    );
}

#[test]
fn check_draw_many() {
    run_stdlib_test(
        r#"operation Test() : Result[] {
            use qubit = Qubit();
            let results = Microsoft.Quantum.Arrays.DrawMany(q => {X(q); M(q)}, 3, qubit);
            Reset(qubit);
            results
        }"#,
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(false),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_excluding() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Excluding([], [])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Excluding([], [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Int(10),
                Value::Int(11),
                Value::Int(12),
                Value::Int(13),
                Value::Int(14),
                Value::Int(15),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Excluding([1, 3, 4], [10, 11, 12, 13, 14, 15])",
        &Value::Array(vec![Value::Int(10), Value::Int(12), Value::Int(15)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Excluding([3, 1, 4, 1], [10, 11, 12, 13, 14, 15])",
        &Value::Array(vec![Value::Int(10), Value::Int(12), Value::Int(15)].into()),
    );
}

#[test]
fn check_enumerated() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Enumerated([false, true, false])",
        &Value::Array(
            vec![
                Value::Tuple(vec![Value::Int(0), Value::Bool(false)].into()),
                Value::Tuple(vec![Value::Int(1), Value::Bool(true)].into()),
                Value::Tuple(vec![Value::Int(2), Value::Bool(false)].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_fold() {
    run_stdlib_test(
        r#"operation Test() : Int {
            Microsoft.Quantum.Arrays.Fold((x, y) -> x + y, 0, [1, 2, 3, 4, 5])
        }"#,
        &Value::Int(15),
    );
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.Fold((x, y) -> x or y, false, [true, false, true])
        }"#,
        &Value::Bool(true),
    );
    run_stdlib_test(
        r#"operation Test() : Bool {
            Microsoft.Quantum.Arrays.Fold((x, y) -> x and y, true, [true, false, true])
        }"#,
        &Value::Bool(false),
    );
}

#[test]
fn check_for_each() {
    run_stdlib_test(
        r#"operation Test() : Result[] {
            use register = Qubit[3];
            Microsoft.Quantum.Arrays.ForEach
                (q => {X(q); Microsoft.Quantum.Measurement.MResetZ(q)},
                register)
        }"#,
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_head() {
    run_stdlib_test_expression("Microsoft.Quantum.Arrays.Head([5,6,7,8])", &Value::Int(5));
}

#[test]
fn check_index_range() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Start",
        &Value::Int(0),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Step",
        &Value::Int(1),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::End",
        &Value::Int(3),
    );
}

#[test]
fn check_interleaved() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Interleaved([1, 2, 3], [-1, -2, -3])",
        &Value::Array(
            vec![
                Value::Int(1),
                Value::Int(-1),
                Value::Int(2),
                Value::Int(-2),
                Value::Int(3),
                Value::Int(-3),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Interleaved([true, true], [false])",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)].into()),
    );
}

#[test]
fn check_is_empty() {
    run_stdlib_test_expression("Microsoft.Quantum.Arrays.IsEmpty([])", &Value::Bool(true));
    run_stdlib_test_expression("Microsoft.Quantum.Arrays.IsEmpty([1])", &Value::Bool(false));
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsEmpty([1, 2, 3, 4, 5])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_rectangular_array() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1]])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2], [3, 4]])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2, 3], [4, 5, 6]])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2], [3, 4, 5]])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_square_array() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1]])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2], [3, 4]])",
        &Value::Bool(true),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2, 3], [4, 5, 6]])",
        &Value::Bool(false),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2], [3, 4], [5, 6]])",
        &Value::Bool(false),
    );
}

#[test]
fn check_mapped() {
    run_stdlib_test(
        r#"operation Test() : Int[] {
            Microsoft.Quantum.Arrays.Mapped(i -> i * 2, [0, 1, 2])
        }"#,
        &Value::Array(vec![Value::Int(0), Value::Int(2), Value::Int(4)].into()),
    );
}

#[test]
fn check_mapped_by_index() {
    run_stdlib_test(
        r#"operation Test() : Bool[] {
            Microsoft.Quantum.Arrays.MappedByIndex((index, element) -> index == element ,[0, -1, 2])
        }"#,
        &Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)].into()),
    );
}

#[test]
fn check_most() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Most([5,6,7,8])",
        &Value::Array(vec![Value::Int(5), Value::Int(6), Value::Int(7)].into()),
    );
}

#[test]
fn check_padded() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Padded(-5, 2, [10, 11, 12])",
        &Value::Array(
            vec![
                Value::Int(10),
                Value::Int(11),
                Value::Int(12),
                Value::Int(2),
                Value::Int(2),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Padded(5, 2, [10, 11, 12])",
        &Value::Array(
            vec![
                Value::Int(2),
                Value::Int(2),
                Value::Int(10),
                Value::Int(11),
                Value::Int(12),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_partitioned() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Partitioned([2, 1], [2, 3, 5, 7])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(5)].into()),
                Value::Array(vec![Value::Int(7)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Partitioned([2, 2], [2, 3, 5, 7])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(5), Value::Int(7)].into()),
                Value::Array(vec![].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_sequence_i() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.SequenceI(0, 3)",
        &Value::Array(vec![Value::Int(0), Value::Int(1), Value::Int(2), Value::Int(3)].into()),
    );
}

#[test]
fn check_rest() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Rest([5,6,7,8])",
        &Value::Array(vec![Value::Int(6), Value::Int(7), Value::Int(8)].into()),
    );
}

#[test]
fn check_reversed() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Reversed([5,6,7,8])",
        &Value::Array(vec![Value::Int(8), Value::Int(7), Value::Int(6), Value::Int(5)].into()),
    );
}

#[test]
fn check_subarray() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Subarray([3, 0, 2, 1], [1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(4), Value::Int(1), Value::Int(3), Value::Int(2)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Subarray([1, 2, 2], [1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(2), Value::Int(3), Value::Int(3)].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Subarray([0, 0, 0, 0, 0], [false])",
        &Value::Array(
            vec![
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_tail() {
    run_stdlib_test_expression("Microsoft.Quantum.Arrays.Tail([5,6,7,8])", &Value::Int(8));
}

#[test]
fn check_transposed() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Transposed([[1, 2, 3], [4, 5, 6]])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1), Value::Int(4)].into()),
                Value::Array(vec![Value::Int(2), Value::Int(5)].into()),
                Value::Array(vec![Value::Int(3), Value::Int(6)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Transposed([[1, 4], [2, 5], [3, 6]])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(4), Value::Int(5), Value::Int(6)].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_unzipped() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Unzipped([])",
        &Value::Tuple(vec![Value::Array(vec![].into()), Value::Array(vec![].into())].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Unzipped([(5, true), (4, false), (3, true), (2, true), (1, false)])",
        &Value::Tuple(
            vec![
                Value::Array(vec![Value::Int(5), Value::Int(4), Value::Int(3), Value::Int(2), Value::Int(1)].into()),
                Value::Array(
                    vec![
                        Value::Bool(true),
                        Value::Bool(false),
                        Value::Bool(true),
                        Value::Bool(true),
                        Value::Bool(false)
                    ]
                    .into()
                ),
            ]
            .into()
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Unzipped([(true, 5), (false, 4), (true, 3), (true, 2), (false, 1)])",
        &Value::Tuple(
            vec![
                Value::Array(
                    vec![
                        Value::Bool(true),
                        Value::Bool(false),
                        Value::Bool(true),
                        Value::Bool(true),
                        Value::Bool(false)
                    ]
                    .into()
                ),
                Value::Array(vec![Value::Int(5), Value::Int(4), Value::Int(3), Value::Int(2), Value::Int(1)].into()),
            ]
            .into()
        ),
    );
}

#[test]
fn check_windows() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Windows(1, [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1)].into()),
                Value::Array(vec![Value::Int(2)].into()),
                Value::Array(vec![Value::Int(3)].into()),
                Value::Array(vec![Value::Int(4)].into()),
                Value::Array(vec![Value::Int(5)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Windows(3, [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(2), Value::Int(3), Value::Int(4)].into()),
                Value::Array(vec![Value::Int(3), Value::Int(4), Value::Int(5)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Windows(5, [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![Value::Array(
                vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5),
                ]
                .into(),
            )]
            .into(),
        ),
    );
}

#[test]
fn check_zipped() {
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([], [])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([1], [])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([], [false])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([1, 2, 3, 4, 5], [false, true, true, false, true])",
        &Value::Array(
            vec![
                Value::Tuple(vec![Value::Int(1), Value::Bool(false)].into()),
                Value::Tuple(vec![Value::Int(2), Value::Bool(true)].into()),
                Value::Tuple(vec![Value::Int(3), Value::Bool(true)].into()),
                Value::Tuple(vec![Value::Int(4), Value::Bool(false)].into()),
                Value::Tuple(vec![Value::Int(5), Value::Bool(true)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([false, true, true, false, true], [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![
                Value::Tuple(vec![Value::Bool(false), Value::Int(1)].into()),
                Value::Tuple(vec![Value::Bool(true), Value::Int(2)].into()),
                Value::Tuple(vec![Value::Bool(true), Value::Int(3)].into()),
                Value::Tuple(vec![Value::Bool(false), Value::Int(4)].into()),
                Value::Tuple(vec![Value::Bool(true), Value::Int(5)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([1, 2, 3], [false, true, true, false, true])",
        &Value::Array(
            vec![
                Value::Tuple(vec![Value::Int(1), Value::Bool(false)].into()),
                Value::Tuple(vec![Value::Int(2), Value::Bool(true)].into()),
                Value::Tuple(vec![Value::Int(3), Value::Bool(true)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test_expression(
        "Microsoft.Quantum.Arrays.Zipped([1, 2, 3, 4, 5], [false, true, true])",
        &Value::Array(
            vec![
                Value::Tuple(vec![Value::Int(1), Value::Bool(false)].into()),
                Value::Tuple(vec![Value::Int(2), Value::Bool(true)].into()),
                Value::Tuple(vec![Value::Int(3), Value::Bool(true)].into()),
            ]
            .into(),
        ),
    );
}
