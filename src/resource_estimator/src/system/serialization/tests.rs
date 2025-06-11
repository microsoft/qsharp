// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::parse_time_str_as_ns;

#[test]
fn test_time_parsing() {
    assert_eq!(parse_time_str_as_ns("1 ns"), Some(1));
    assert_eq!(parse_time_str_as_ns("1.0 ns"), Some(1));
    assert_eq!(parse_time_str_as_ns("1.0 ns"), Some(1));
    assert_eq!(parse_time_str_as_ns("1e0 ns"), Some(1));
    assert_eq!(parse_time_str_as_ns("1e1 ns"), Some(10));
    assert_eq!(parse_time_str_as_ns("1e-1 ns"), Some(0));
    assert_eq!(parse_time_str_as_ns("+1e-1 ns"), Some(0));
    assert!(parse_time_str_as_ns("-1e-1 ns").is_none());

    assert_eq!(parse_time_str_as_ns("1 μs"), Some(1000));
    assert_eq!(parse_time_str_as_ns("1 µs"), Some(1000));
    assert_eq!(parse_time_str_as_ns("1 us"), Some(1000));
    assert_eq!(parse_time_str_as_ns("1 \u{03bc}s"), Some(1000));
    assert_eq!(parse_time_str_as_ns("1 ms"), Some(1_000_000));
    assert_eq!(parse_time_str_as_ns("1 s"), Some(1_000_000_000));
}
