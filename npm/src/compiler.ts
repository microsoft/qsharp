// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../lib/node/qsc_wasm.cjs")

export class Compiler {
    wasm: Wasm;

    constructor(wasm: Wasm) {
        this.wasm = wasm;
    }

    runShot() {
        return this.wasm.run("test", "expr", () => {}, 5);
    }
}
