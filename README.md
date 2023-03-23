# Q# language and tooling:

> Under development

This repository contains Q# tooling, specifically:

- **[compiler](./compiler/qsc/)**: command line compiler
- **[library](./library/)**: standard library
- **[npm](./npm/)**: Q# npm module
- **[playground](./playground/)**: simple website for interacting with Q#


## Building

To build this repository there are 4 dependencies that need to be installed. These are:

- Python (version 3.11 or later. See <https://python.org>)
- Rust (version 1.65 or later. See <https://www.rust-lang.org/tools/install>)
- Node.js (version 16.17 or later. See <https://nodejs.org/>)
- wasm-pack (version 0.10 or later. See <https://rustwasm.github.io/wasm-pack/installer/>)

The build script will check these dependencies and their versions and fail if not met.

To build, in the root directory run `python ./build.py`. By default this will run a development
build of each project.

To run the "playground" locally, `cd` into the `playground` directory, and run `npm start`.
This will launch a local web server and output the URL to visit to the console.

## Feedback

If you have feedback about the content in this repository, please let us know by filing a [new issue](https://github.com/microsoft/qsharp/issues/new/choose)!

## Reporting Security Issues

Security issues and bugs should be reported privately following our [security issue documentation](./SECURITY.md#reporting-security-issues).

## Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

For more details, please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Legal and Licensing

### Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft 
trademarks or logos is subject to and must follow 
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
