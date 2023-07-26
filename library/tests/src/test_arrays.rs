// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_expression;
use num_bigint::BigInt;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Arrays namespace

#[test]
fn check_all() {
    test_expression(
        "Microsoft.Quantum.Arrays.All(x -> x != 0, [1, 2, 3, 4, 5])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.All(x -> x != 0, [1, 2, 0, 4, 5])",
        &Value::Bool(false),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.All(x -> x == One, [One, One, One])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.All(x -> x == One, [One, One, Zero])",
        &Value::Bool(false),
    );
}

#[test]
fn check_any() {
    test_expression(
        "Microsoft.Quantum.Arrays.Any(x -> x % 2 == 0, [1, 3, 6, 7, 9])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Any(x -> x % 2 == 0, [1, 3, 5, 7, 9])",
        &Value::Bool(false),
    );
}

#[test]
fn check_chunks() {
    test_expression(
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
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.Chunks(2, empty)
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Chunks(2, [10])",
        &Value::Array(vec![Value::Array(vec![Value::Int(10)].into())].into()),
    );
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.Chunks(3, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10), Value::Int(11), Value::Int(12)].into()),
                Value::Array(vec![Value::Int(13), Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.ColumnAt(0, [[1, 2, 3], [4, 5, 6], [7, 8, 9]])",
        &Value::Array(vec![Value::Int(1), Value::Int(4), Value::Int(7)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.ColumnAt(2, [[true, true, true], [false, false, false]])",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false)].into()),
    );
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.Count(x -> x % 2 != 0, [1, 3, 6, 7, 9])",
        &Value::Int(4),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Count(x -> x % 2 == 0, [1, 3, 6, 7, 9])",
        &Value::Int(1),
    );
}

#[test]
fn check_diagnonal() {
    test_expression(
        "{
            let empty: Int[][] = [];
            Microsoft.Quantum.Arrays.Diagonal(empty)
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1]])",
        &Value::Array(vec![Value::Int(1)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2, 3], [4, 5, 6], [7, 8, 9]])",
        &Value::Array(vec![Value::Int(1), Value::Int(5), Value::Int(9)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2, 3], [4, 5, 6]])",
        &Value::Array(vec![Value::Int(1), Value::Int(5)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2], [3, 4], [5, 6]])",
        &Value::Array(vec![Value::Int(1), Value::Int(4)].into()),
    );
}

#[test]
fn check_draw_many() {
    test_expression(
        "{
            use qubit = Qubit();
            let results = Microsoft.Quantum.Arrays.DrawMany(q => {X(q); M(q)}, 3, qubit);
            Reset(qubit);
            results
        }",
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
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.Excluding(empty, empty)
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.Excluding([1, 3, 4], [10, 11, 12, 13, 14, 15])",
        &Value::Array(vec![Value::Int(10), Value::Int(12), Value::Int(15)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Excluding([3, 1, 4, 1], [10, 11, 12, 13, 14, 15])",
        &Value::Array(vec![Value::Int(10), Value::Int(12), Value::Int(15)].into()),
    );
}

#[test]
fn check_enumerated() {
    test_expression(
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
fn check_filtered() {
    test_expression(
        "Microsoft.Quantum.Arrays.Filtered(x -> x % 2 == 0, [0, 1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(0), Value::Int(2), Value::Int(4)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Filtered(x -> x % 2 != 0, [1, 2, 3, 4, 5])",
        &Value::Array(vec![Value::Int(1), Value::Int(3), Value::Int(5)].into()),
    );
}

#[test]
fn check_flat_mapped() {
    test_expression(
        "Microsoft.Quantum.Arrays.FlatMapped(x -> Repeated(x, 2), [1, 2, 3])",
        &Value::Array(
            vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(2),
                Value::Int(2),
                Value::Int(3),
                Value::Int(3),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_flattened() {
    test_expression(
        "Microsoft.Quantum.Arrays.Flattened([[1, 2], [3], [4, 5, 6]])",
        &Value::Array(
            vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
                Value::Int(6),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_fold() {
    test_expression(
        "Microsoft.Quantum.Arrays.Fold((x, y) -> x + y, 0, [1, 2, 3, 4, 5])",
        &Value::Int(15),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Fold((x, y) -> x or y, false, [true, false, true])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Fold((x, y) -> x and y, true, [true, false, true])",
        &Value::Bool(false),
    );
}

#[test]
fn check_for_each() {
    test_expression(
        "{
            use register = Qubit[3];
            Microsoft.Quantum.Arrays.ForEach
                (q => {X(q); Microsoft.Quantum.Measurement.MResetZ(q)},
                register)
        }",
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
    test_expression("Microsoft.Quantum.Arrays.Head([5,6,7,8])", &Value::Int(5));
}

#[test]
fn check_head_and_rest() {
    test_expression(
        "Microsoft.Quantum.Arrays.HeadAndRest([5,6,7,8])",
        &Value::Tuple(
            vec![
                Value::Int(5),
                Value::Array(vec![Value::Int(6), Value::Int(7), Value::Int(8)].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_index_of() {
    test_expression(
        "Microsoft.Quantum.Arrays.IndexOf(x -> x % 2 != 0, [10, 8, 6, 5, 4])",
        &Value::Int(3),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IndexOf(x -> x % 2 == 0, [1, 3, 4, 5, 7])",
        &Value::Int(2),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IndexOf(x -> x % 2 == 0, [1, 3, 5, 7, 9])",
        &Value::Int(-1),
    );
}

#[test]
fn check_index_range() {
    test_expression(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Start",
        &Value::Int(0),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Step",
        &Value::Int(1),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::End",
        &Value::Int(3),
    );
}

#[test]
fn check_interleaved() {
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.Interleaved([true, true], [false])",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)].into()),
    );
}

#[test]
fn check_is_empty() {
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.IsEmpty(empty)
        }",
        &Value::Bool(true),
    );
    test_expression("Microsoft.Quantum.Arrays.IsEmpty([1])", &Value::Bool(false));
    test_expression(
        "Microsoft.Quantum.Arrays.IsEmpty([1, 2, 3, 4, 5])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_rectangular_array() {
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.IsRectangularArray([empty])
        }",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1]])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2], [3, 4]])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2, 3], [4, 5, 6]])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2], [3, 4, 5]])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_sorted() {
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.IsSorted((x, y) -> x <= y, empty)
        }",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSorted((x, y) -> x <= y, [1])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSorted((x, y) -> x <= y, [1, 2, 3, 4, 5])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSorted((x, y) -> x >= y, [5, 4, 3, 2, 1])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSorted((x, y) -> x <= y, [1, 2, 3, 5, 4])",
        &Value::Bool(false),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSorted((x, y) -> x <= y, [5, 4, 3, 2, 1])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_square_array() {
    test_expression(
        "{
            let empty: Int[][] = [];
            Microsoft.Quantum.Arrays.IsSquareArray(empty)
        }",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1]])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2], [3, 4]])",
        &Value::Bool(true),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2, 3], [4, 5, 6]])",
        &Value::Bool(false),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2], [3, 4], [5, 6]])",
        &Value::Bool(false),
    );
}

#[test]
fn check_mapped() {
    test_expression(
        "Microsoft.Quantum.Arrays.Mapped(i -> i * 2, [0, 1, 2])",
        &Value::Array(vec![Value::Int(0), Value::Int(2), Value::Int(4)].into()),
    );
}

#[test]
fn check_mapped_by_index() {
    test_expression(
        "Microsoft.Quantum.Arrays.MappedByIndex((index, element) -> index == element ,[0, -1, 2])",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)].into()),
    );
}

#[test]
fn check_mapped_over_range() {
    test_expression(
        "Microsoft.Quantum.Arrays.MappedOverRange(x -> x + 1, 0..2..10)",
        &Value::Array(
            vec![
                Value::Int(1),
                Value::Int(3),
                Value::Int(5),
                Value::Int(7),
                Value::Int(9),
                Value::Int(11),
            ]
            .into(),
        ),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.MappedOverRange(x -> x * 2, 3..-1..1)",
        &Value::Array(vec![Value::Int(6), Value::Int(4), Value::Int(2)].into()),
    );
}

#[test]
fn check_most() {
    test_expression(
        "Microsoft.Quantum.Arrays.Most([5, 6, 7, 8])",
        &Value::Array(vec![Value::Int(5), Value::Int(6), Value::Int(7)].into()),
    );
}

#[test]
fn check_most_and_tail() {
    test_expression(
        "Microsoft.Quantum.Arrays.MostAndTail([5, 6, 7, 8])",
        &Value::Tuple(
            vec![
                Value::Array(vec![Value::Int(5), Value::Int(6), Value::Int(7)].into()),
                Value::Int(8),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_padded() {
    test_expression(
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
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.Padded(3, 2, [10, 11, 12])",
        &Value::Array(vec![Value::Int(10), Value::Int(11), Value::Int(12)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Padded(-3, 2, [10, 11, 12])",
        &Value::Array(vec![Value::Int(10), Value::Int(11), Value::Int(12)].into()),
    );
}

#[test]
fn check_partitioned() {
    test_expression(
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
    test_expression(
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
    test_expression(
        "Microsoft.Quantum.Arrays.SequenceI(0, 3)",
        &Value::Array(vec![Value::Int(0), Value::Int(1), Value::Int(2), Value::Int(3)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.SequenceI(-5, -2)",
        &Value::Array(
            vec![
                Value::Int(-5),
                Value::Int(-4),
                Value::Int(-3),
                Value::Int(-2),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_sequence_l() {
    test_expression(
        "Microsoft.Quantum.Arrays.SequenceL(0L, 3L)",
        &Value::Array(
            vec![
                Value::BigInt(BigInt::from(0)),
                Value::BigInt(BigInt::from(1)),
                Value::BigInt(BigInt::from(2)),
                Value::BigInt(BigInt::from(3)),
            ]
            .into(),
        ),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.SequenceL(-5L, -2L)",
        &Value::Array(
            vec![
                Value::BigInt(BigInt::from(-5)),
                Value::BigInt(BigInt::from(-4)),
                Value::BigInt(BigInt::from(-3)),
                Value::BigInt(BigInt::from(-2)),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_sorted() {
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.Sorted((x, y) -> x <= y, empty)
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Sorted((x, y) -> x <= y, [-1])",
        &Value::Array(vec![Value::Int(-1)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Sorted((x, y) -> x <= y, [1, 2, 0, 4, 3])",
        &Value::Array(
            vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ]
            .into(),
        ),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Sorted((x, y) -> x >= y, [1, 2, 0, 4, 3])",
        &Value::Array(
            vec![
                Value::Int(4),
                Value::Int(3),
                Value::Int(2),
                Value::Int(1),
                Value::Int(0),
            ]
            .into(),
        ),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Sorted((x, y) -> x <= y, [-1, 2, 0, 1, -2])",
        &Value::Array(
            vec![
                Value::Int(-2),
                Value::Int(-1),
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_rest() {
    test_expression(
        "Microsoft.Quantum.Arrays.Rest([5,6,7,8])",
        &Value::Array(vec![Value::Int(6), Value::Int(7), Value::Int(8)].into()),
    );
}

#[test]
fn check_reversed() {
    test_expression(
        "Microsoft.Quantum.Arrays.Reversed([5,6,7,8])",
        &Value::Array(vec![Value::Int(8), Value::Int(7), Value::Int(6), Value::Int(5)].into()),
    );
}

#[test]
fn check_subarray() {
    test_expression(
        "Microsoft.Quantum.Arrays.Subarray([3, 0, 2, 1], [1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(4), Value::Int(1), Value::Int(3), Value::Int(2)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Subarray([1, 2, 2], [1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(2), Value::Int(3), Value::Int(3)].into()),
    );
    test_expression(
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
fn check_swapped() {
    test_expression(
        "Microsoft.Quantum.Arrays.Swapped(1, 3, [0, 1, 2, 3, 4])",
        &Value::Array(
            vec![
                Value::Int(0),
                Value::Int(3),
                Value::Int(2),
                Value::Int(1),
                Value::Int(4),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_tail() {
    test_expression("Microsoft.Quantum.Arrays.Tail([5,6,7,8])", &Value::Int(8));
}

#[test]
fn check_transposed() {
    test_expression(
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
    test_expression(
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
    test_expression(
        "{
            let empty: (Int, Int)[] = [];
            Microsoft.Quantum.Arrays.Unzipped(empty)
        }",
        &Value::Tuple(vec![Value::Array(vec![].into()), Value::Array(vec![].into())].into()),
    );
    test_expression(
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
    test_expression(
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
fn check_where() {
    test_expression(
        "Microsoft.Quantum.Arrays.Where(x -> x % 2 == 0, [0, 1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(0), Value::Int(2), Value::Int(4)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Arrays.Where(x -> x % 2 != 0, [1, 2, 3, 4, 5])",
        &Value::Array(vec![Value::Int(0), Value::Int(2), Value::Int(4)].into()),
    );
}

#[test]
fn check_windows() {
    test_expression(
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
    test_expression(
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
    test_expression(
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
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.Zipped(empty, empty)
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.Zipped([1], empty)
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
        "{
            let empty: Int[] = [];
            Microsoft.Quantum.Arrays.Zipped(empty, [false])
        }",
        &Value::Array(vec![].into()),
    );
    test_expression(
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
    test_expression(
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
    test_expression(
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
    test_expression(
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
