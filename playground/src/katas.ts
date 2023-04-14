import {eventStringToMsg, getAllKatas, getKata, renderDump, runExercise, type Kata, type Exercise} from "qsharp/browser";

// TODO (cesarzc): should probably be in the npm package.
interface VerificationResult {
    kind: "VerificationResult";
    result: boolean;
}

interface KataError {
    kind: "KataError";
    error: string;
}

interface RuntimeError {
    kind: "RuntimeError";
    error: string;
}

interface UnexpectedError {
    kind: "UnexpectedError";
    error: string;
}

type KataOutput = VerificationResult | KataError | RuntimeError | UnexpectedError;

function clearDiv(div: HTMLDivElement) {
    while (div.hasChildNodes()) {
        div.removeChild(div.lastChild!);
    }
}

function renderKataOutput(output: KataOutput) : HTMLDivElement {
    let outputDiv = document.createElement("div");
    if (output.kind === "VerificationResult") {
        outputDiv.textContent = `Kata Verification: ${output.result}`;
    } else if (output.kind === "KataError") {
        outputDiv.textContent = `${output.kind}: ${output.error}`;
    }

    return outputDiv;
}

function renderExercise(exercise: Exercise) : HTMLDivElement {
    let exerciseDiv = document.createElement("div");
    exerciseDiv.className = "kata-exercise";
    let exerciseContent = document.createElement("div");
    exerciseContent.innerHTML = exercise.contentAsHtml;
    exerciseDiv.append(exerciseContent);
    let sourceCodeArea = document.createElement("textarea");
    sourceCodeArea.id = `source_${exercise.id}`;
    sourceCodeArea.rows = 30;
    sourceCodeArea.cols = 80;
    sourceCodeArea.value = exercise.placeholderImplementation;
    exerciseDiv.append(sourceCodeArea);
    let outputDiv = document.createElement("div");
    outputDiv.id = `ouput_${exercise.id}`;
    exerciseDiv.append(outputDiv);
    let verifyButtonDiv = document.createElement("div");
    exerciseDiv.append(verifyButtonDiv);
    let verifyButton = document.createElement("button");
    verifyButton.textContent = "Verify";
    verifyButton.id = `verify_${exercise.id}`;
    verifyButtonDiv.append(verifyButton);

    //
    let kataSimulationCallback = (ev: string) => {
        let result = eventStringToMsg(ev);
        if (!result) {
            console.error("Unrecognized message: " + ev);
            return;
        }
        let paragraph = document.createElement('p') as HTMLParagraphElement;
        console.log(`Callback invoked: ${result.type}`);
        switch (result.type) {
            case "Result":
                paragraph.textContent = `RESULT: ${result.result}`;
                break;
            case "Message":
                paragraph.textContent = `MESSAGE: ${result.message}`;
                break;
            case "DumpMachine":
                let table = document.createElement("table");
                table.innerHTML = renderDump(result.state);
                paragraph.appendChild(table);
                break;
        }

        outputDiv.append(paragraph);
    }

    //
    verifyButton.addEventListener('click', _ => {
        clearDiv(outputDiv);
        let exerciseImplementation = sourceCodeArea.value;
        try {
            let result = runExercise(exercise.id, exerciseImplementation, kataSimulationCallback);
            let verificationResult: VerificationResult = {kind: "VerificationResult", result: result};
            let renderedResult = renderKataOutput(verificationResult);
            outputDiv.prepend(renderedResult);
        } catch(e)
        {
            if (e instanceof Error) {
                let kataError: KataError = {kind: "KataError", error: e.message};
                let renderedError = renderKataOutput(kataError);
                outputDiv.prepend(renderedError);
            }
        }
    });

    return exerciseDiv;
}

function renderKata(kata: Kata) : HTMLDivElement {
    let kataDiv = document.createElement("div");

    // Render the content.
    let kataContent = document.createElement("div");
    kataContent.innerHTML = kata.contentAsHtml;
    kataDiv.append(kataContent);

    // Render each one of the exercises.
    for (let exercise of kata.exercises)
    {
        let renderedExercise = renderExercise(exercise);
        kataDiv.append(renderedExercise);
    }
    return kataDiv;
}

export function RenderKatas() {
    // Katas are rendered inside a div element with "katas-canvas" as id.
    let canvasDiv = document.querySelector('#katas-canvas') as HTMLDivElement;

    // Clear the katas' canvas every time before re-rendering.
    clearDiv(canvasDiv);

    // Render the selected kata.
    let katasDropdown = document.querySelector('#katas-list') as HTMLSelectElement;
    let selectedOption = katasDropdown.item(katasDropdown.selectedIndex)!;
    let kata = getKata(selectedOption.value);
    let renderedKata = renderKata(kata);
    canvasDiv.append(renderedKata);
}

export function PopulateKatasList() {
    let katasDropdown = document.querySelector('#katas-list') as HTMLSelectElement;
    for (let kata of getAllKatas())
    {
        let option = document.createElement("option");
        option.value = kata.id;
        option.text = kata.title;
        katasDropdown.add(option);
    }
}