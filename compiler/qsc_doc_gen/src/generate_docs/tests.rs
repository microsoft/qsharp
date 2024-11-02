// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::generate_docs;
use expect_test::expect;

#[test]
fn generates_standard_item() {
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
        qsharp.package: __Core__
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

#[test]
fn generates_unrestricted_item() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "Std.Diagnostics/CheckZero.md")
        .expect("Could not file doc file for Length");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"
        ---
        uid: Qdk.Std.Diagnostics.CheckZero
        title: CheckZero operation
        ms.date: {TIMESTAMP}
        ms.topic: managed-reference
        qsharp.kind: operation
        qsharp.package: __Std__
        qsharp.namespace: Std.Diagnostics
        qsharp.name: CheckZero
        qsharp.summary: "Checks whether a qubit is in the |0⟩ state, returning true if it is."
        ---

        # CheckZero operation

        Fully qualified name: Std.Diagnostics.CheckZero

        ```qsharp
        operation CheckZero(qubit : Qubit) : Bool
        ```

        ## Summary
        Checks whether a qubit is in the |0⟩ state, returning true if it is.

        ## Description
        This operation checks whether a qubit is in the |0⟩ state. It will return true only
        if the qubit is deterministically in the |0⟩ state, and will return false otherwise. This operation
        does not change the state of the qubit.

        ## Input
        ### qubit
        The qubit to check.
        ## Output
        True if the qubit is in the |0⟩ state, false otherwise.

        ## Remarks
        This operation is useful for checking whether a qubit is in the |0⟩ state during simulation. It is not possible to check
        this on hardware without measuring the qubit, which could change the state.
    "#]]
    .assert_eq(full_contents.as_str());
}
