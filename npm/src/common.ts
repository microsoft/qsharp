// Each DumpMachine output is represented as an object where each key is a basis
// state, e.g., "|3>" and the value is the [real, imag] parts of the complex amplitude.
export type Dump = {
    [index: string]: [number, number];
};

export interface VSDiagnostic {
    start_pos: number;
    end_pos: number;
    message: string;
    severity: number;
}

export interface DumpMsg {
    type: "DumpMachine";
    state: Dump;
}

export interface MessageMsg {
    type: "Message";
    message: string;
}

export interface ResultMsg {
    type: "Result";
    success: boolean;
    result: string | VSDiagnostic;
}

export type ShotResult = {
    success: boolean;
    result: string | VSDiagnostic;
    events: Array<MessageMsg | DumpMsg>;
}

export type EventMsg = ResultMsg | DumpMsg | MessageMsg;

export function outputAsResult(msg: string) : ResultMsg | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "Result" && typeof obj.success == "boolean") {
            return {type: "Result", success: obj.success, result: obj.result};
        }
    } catch {
        return null;
    }
    return null;
}

export function outputAsMessage(msg: string) : MessageMsg | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "Message" && typeof obj.message == "string") {
            return {type: "Message", message: obj.message};
        }
    } catch {
        return null;
    }
    return null;
}

export function outputAsDump(msg: string) : DumpMsg | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "DumpMachine" && typeof obj.state == "object") {
            return { type: "DumpMachine", state: obj.state };
        }
    } catch {
        return null;
    }
    return null;
}

export function eventStringToMsg(msg: string) : EventMsg | null {
    return outputAsResult(msg) || outputAsMessage(msg) || outputAsDump(msg);
}

export type RunFn = (code: string, expr: string, event_cb: Function, shots: number) => void;

export function run_shot_internal(code: string, expr: string, run: RunFn) : ShotResult {
    let result : ShotResult = {
        success: false,
        result: "pending",
        events: [],
    };

    run(code, expr, (msg:string) => {
        let eventObj = eventStringToMsg(msg);
        if (!eventObj) return;

        switch (eventObj.type) {
            case "Result":
                result.success = eventObj.success;
                result.result = eventObj.result;
                break;
            default:
                result.events.push(eventObj);
                break;
        }
    }, 1);

    return result;
}