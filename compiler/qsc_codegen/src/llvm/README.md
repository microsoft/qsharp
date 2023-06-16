# LLVM Rust Types Port

This is a port of a small subset of the [Rust llvm-ir](https://github.com/cdisselkoen/llvm-ir) crate to provide a
Rust types for LLVM structures. Here, instead of populating the contents via calls into LLVM native libraries, it avoids any actual
LLVM dependency by building up the types and using `Display` trait implementations to output the correspond IR in .ll format.
This code is not intended for use outside of the Q# compiler code generation component.

Please use [llvm-ir](https://github.com/cdisselkoen/llvm-ir) for the official release and to make any contributions.
