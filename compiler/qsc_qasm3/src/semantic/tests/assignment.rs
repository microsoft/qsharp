// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use super::check;

#[test]
#[ignore = "Not yet implemented"]
fn too_many_indicies_in_indexed_assignment() {
    check(
        r#"
        array[float[32], 3, 2] multiDim = {{1.1, 1.2}, {2.1, 2.2}, {3.1, 3.2}};
        multiDim[1, 1, 3] = 2.3;
        "#,
        &expect![[r#""#]],
    );
}
