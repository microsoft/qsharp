# Playground

This is a simple web site built using the Monaco editor and the qsharp npm package.

## Building Playground Locally

1. Build the entire repo by running `./build.py` in the root directory.
If you only want to build the functionality necessary to run the playground, you can use `python .\build.py --wasm --npm --play`.
2. Then make `./playground` your current directory and run `npm start` to start the web server.
3. Copy the URL that will be printed to console and open it in a browser to use the playground.
