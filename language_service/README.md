# Q# Language Service

This crate contains the implementation of Q# editor features such as
auto-complete, go-to-definition and hover.

The interface for the language service is based on the
[Language Server Protocol](https://microsoft.github.io/language-server-protocol/specifications/specification-current),
even though a true LSP server implementation is not provided here.
Following the LSP protocol makes it easy to use the implementation in
a variety of editors (Monaco, VS Code, JupyterLab) whose extension APIs
either use LSP or map closely to LSP concepts.
