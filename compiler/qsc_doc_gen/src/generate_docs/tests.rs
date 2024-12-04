// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::generate_docs;
use expect_test::expect;

#[test]
fn docs_generation() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "Std.Core/Length.md")
        .expect("Could not file doc file for Length");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"
        ---
        uid: Qdk.Std.Core.Length
        title: Length function
        ms.date: {TIMESTAMP}
        ms.topic: managed-reference
        qsharp.kind: function
        qsharp.namespace: Std.Core
        qsharp.name: Length
        qsharp.summary: "Returns the number of elements in the input array `a`."
        ---

        # Length function

        Fully qualified name: Std.Core.Length

        ```qsharp
        function Length<'T>(a : 'T[]) : Int
        ```

        ## Summary
        Returns the number of elements in the input array `a`.

        ## Input
        ### a
        Input array.

        ## Output
        The total number of elements in the input array `a`.

        ## Example
        ```qsharp
        Message($"{ Length([0, 0, 0]) }"); // Prints 3
        ```
    "#]]
    .assert_eq(full_contents.as_str());
}
