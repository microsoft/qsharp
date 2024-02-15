// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use regex_lite::Regex;

pub(crate) const fn f64_nan() -> f64 {
    f64::NAN
}

/// This parses simple numbers (assuming nanoseconds) or strings with suffixes s
/// (seconds), ms (milliseconds), μs (microseconds), and ns (nanoseconds).  The
/// deserialized return value is a `f64` value representing the time in
/// nanoseconds.
pub(crate) fn parse_time_str_as_ns(v: &str) -> Option<u64> {
    let re = Regex::new(r"^(\+?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?)\s*(s|ms|μs|µs|us|ns)$")
        .expect("regular expression should be valid");

    if let Some(captures) = re.captures(v) {
        let number = captures[1]
            .parse::<f64>()
            .expect("regular expression should enforce f64 formatted string");
        let multiplier = match &captures[3] {
            "s" => 1e9,
            "ms" => 1e6,
            "μs" | "µs" | "us" => 1e3,
            _ => 1.0, // this must be ns (because of the regular expression)
        };

        Some((number * multiplier).round() as u64)
    } else {
        None
    }
}

pub(crate) mod time {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::parse_time_str_as_ns;

    /// A custom serializer for times.
    ///
    /// This adds the `ns` prefix to all time values and serializes them as a
    /// string.
    #[allow(clippy::trivially_copy_pass_by_ref)] // forced by serde interface
    pub(crate) fn serialize<S: Serializer>(
        time: &Option<u64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if let Some(time) = time {
            serializer.serialize_str(&format!("{time} ns"))
        } else {
            serializer.serialize_none()
        }
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        if let Some(result) = parse_time_str_as_ns(&s) {
            Ok(Some(result))
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"cannot parse time",
            ))
        }
    }
}
