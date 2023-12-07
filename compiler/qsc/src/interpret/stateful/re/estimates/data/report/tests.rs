// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{format_duration, format_metric_prefix, format_thousand_sep_f64};

#[test]
fn test_format_thousand_dep() {
    assert_eq!(
        format_thousand_sep_f64(92_592.592_592_592_6),
        String::from("92,592.59")
    );
}

#[test]
fn test_format_prefix() {
    assert_eq!(format_metric_prefix(5_u64), String::from("5"));
    assert_eq!(
        format_metric_prefix(29_734_537_038_u64),
        String::from("29.73G")
    );
    assert_eq!(
        format_metric_prefix(474_209_514_u64),
        String::from("474.21M")
    );
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(6_465_096_000), "6 secs");
    assert_eq!(format_duration(10_920_000), "11 millisecs");
    assert_eq!(format_duration(36_400), "36 microsecs");
    assert_eq!(format_duration(36_499), "36 microsecs");
    assert_eq!(format_duration(36_500), "37 microsecs");
    assert_eq!(format_duration(13_280_983_992_000), "4 hours");
}
