Add a language service configuration called "showTestDiagnostics" .

When true, _every_ document that is open, should produce one info-level diagnostic at the very top, that says the name of the compilation that the file belongs to, as well as its document version.

- single-file projects - the document should report the diagnostic and the compilation it's in (itself)
- multi-file projects - 2 docs exist in a qsharp.json project, both of them, if open, should report the test diagnostic, they should belong to the same compilation
- stdlib - open stdlib files should report the test diagnostic, they should belong to the compilation that caused these documents to be opened
- dependencies - files in dependency packages should report the test diagnostic too, if they're open. they shoudl belong to the same compilation as their dependent project.

Stage 1. Implement the rust bits for this. Write unit test.

Stage 2. Expose through wasm and npm layer. Write unit tests in JS.

Stage 3. Expose this through VS code.

Stage 4. We're going to have the language service VS Code integration tests take advantage of this new pseudo-diagnostic. language service Integration tests should just have this configuration enabled for all the test cases. Don't have specific test cases for the configuration. Now the real magic: The `openDocumentAndWaitForProcessing` helper function can just wait for this diagnostic to appear after opening the document, to confirm that the document was loaded fully by the language service. This would replace the compilatoin status event thing we currently have in that helper.
