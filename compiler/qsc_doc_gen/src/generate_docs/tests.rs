// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::generate_docs;
use expect_test::expect;
use regex_lite::Regex;

#[test]
fn docs_generation() {
    let files = generate_docs();
    let contents = files
        .get("Microsoft.Quantum.Core/Length.md")
        .expect("Could not file doc file for Length");

    let re = Regex::new(r"(ms\.date: ).*(\n)").expect("Invalid regex");
    let temp = re.replace(contents, "$1{TIMESTAMP}$2").to_string();

    expect![[r#"
        ---
        uid: Microsoft.Quantum.Core.Length
        title: Length function
        ms.date: {TIMESTAMP}
        ms.topic: managed-reference
        qsharp.kind: function
        qsharp.namespace: Microsoft.Quantum.Core
        qsharp.name: Length
        qsharp.summary: Returns the number of elements in an array.
        ---

        # Length function

        Namespace: [Microsoft.Quantum.Core](xref:Microsoft.Quantum.Core)

        ```qsharp
        function Length<'T>(a : 'T[]) : Int
        ```

        ## Summary
        Returns the number of elements in an array.

        ## Input
        ### a
        Input array.

        ## Output
        The total count of elements in an array.
    "#]]
    .assert_eq(&temp);
}
