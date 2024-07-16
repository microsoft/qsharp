// # Sample
// Comments
//
// # Description
// Comments begin with two forward slashes (`//`) and continue until the
// end of line. Comments may appear anywhere in the source code.
// Q# does not currently support block comments.
// Documentation comments, or doc comments, are denoted with three
// forward slashes (`///`) instead of two.

import Std.Diagnostics.Result;

/// This is a doc-comment for the `Main` operation.
function Main() : Result[] {
    // Comments can go anywhere in a program, although they typically
    // preface what they refer to.
    return [];
}
