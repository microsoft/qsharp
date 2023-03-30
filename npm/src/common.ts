// Each DumpMachine output is represented as an object where each key is a basis
// state, e.g., "|3>" and the value is the [real, imag] parts of the complex amplitude.
export type Dump = {
    [index: string]: [number, number];
};

export function outputAsMessage(msg: string) : string | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "Message" && typeof obj.message == "string") {
            return obj.message;
        }
    } catch {
        return null;
    }
    return null;
}

export function outputAsDump(msg: string) : Dump | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "DumpMachine" && typeof obj.state == "object") {
            return obj.state as Dump;
        }
    } catch {
        return null;
    }
    return null;
}
