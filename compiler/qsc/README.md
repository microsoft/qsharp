# qsc - Q# command-line compiler

```console
Usage: qsc [OPTIONS] [INPUT]...

Arguments:
  [INPUT]...
          Q# source files to compile, or `-` to read from stdin

Options:
      --nostdlib
          Disable automatic inclusion of the standard library

      --emit <EMIT>
          Emit the compilation unit in the specified format

          Possible values:
          - ast: Abstract syntax tree

      --outdir <DIR>
          Write output to compiler-chosen filename in <dir>

  -v, --verbose
          Enable verbose output

  -e, --entry <ENTRY>
          Entry expression to execute as the main operation

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

# qsi - Q# interactive command-line

```console
Q# Interactive

Usage: qsi [OPTIONS]

Options:
      --use <SOURCES>
          Use the given file on startup as initial session input
      --entry <ENTRY>
          Execute the given Q# expression on startup
      --nostdlib
          Disable automatic inclusion of the standard library
      --exec
          Exit after loading the files or running the given file(s)/entry on the command line
  -q, --qsharp-json <QSHARP_JSON>
          Path to a Q# manifest for a project
  -f, --features <FEATURES>
          Language features to compile with
      --debug
          Compile the given files and interactive snippets in debug mode
  -h, --help
          Print help
  -V, --version
          Print version
```
