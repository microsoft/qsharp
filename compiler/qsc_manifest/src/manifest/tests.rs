use crate::Manifest;
use expect_test::{expect, Expect};
use serde::Deserialize;

pub(super) fn check(input: &str, expect: &Expect) {
    let raw: Result<Manifest, _> = serde_json::from_str(input);
    expect.assert_eq(&format!("{raw:?}"))
}

#[test]
fn parse_empty_manifest() {
    check("{}", &expect!["Ok(Manifest { author: None, license: None, exclude_files: [] })"])
}
