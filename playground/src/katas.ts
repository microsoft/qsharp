import { eventStringToMsg, getAllKatas, getKata, renderDump, runExercise, type Kata, type KataItem, type Exercise } from "qsharp";

// MathJax will already be loaded on the page. Need to call `typeset` when LaTeX content changes.
declare var MathJax: { typeset: () => void; };

interface VerificationResult {
    kind: "VerificationResult";
    result: boolean;
}

interface KataError {
    kind: "KataError";
    error: string;
}

type KataOutput = VerificationResult | KataError;

function renderKataOutput(output: KataOutput): HTMLDivElement {
    let outputDiv = document.createElement("div");
    if (output.kind === "VerificationResult") {
        outputDiv.textContent = `Kata Verification: ${output.result}`;
    } else if (output.kind === "KataError") {
        outputDiv.textContent = `${output.kind}: ${output.error}`;
    }

    return outputDiv;
}

function renderExercise(exercise: Exercise): HTMLDivElement {
    let exerciseDiv = document.createElement("div");
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

    // This callback is the one that processes output produced when running the kata.
    let outputCallback = (ev: string) => {
        let result = eventStringToMsg(ev);
        if (!result) {
            console.error("Unrecognized message: " + ev);
            return;
        }
        let paragraph = document.createElement('p') as HTMLParagraphElement;
        switch (result.type) {
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

    // Run the exercise when clicking the verify button.
    verifyButton.addEventListener('click', async _ => {
        outputDiv.innerHTML = "";
        let exerciseImplementation = sourceCodeArea.value;
        try {
            let result = await runExercise(exercise, exerciseImplementation, outputCallback);
            let verificationResult: VerificationResult = { kind: "VerificationResult", result: result };
            let renderedResult = renderKataOutput(verificationResult);
            outputDiv.prepend(renderedResult);
        } catch (e) {
            if (e instanceof Error) {
                let kataError: KataError = { kind: "KataError", error: e.message };
                let renderedError = renderKataOutput(kataError);
                outputDiv.prepend(renderedError);
            }
        }
    });

    return exerciseDiv;
}

function renderItem(item: KataItem): HTMLDivElement {
    let itemDiv = document.createElement("div");
    itemDiv.className = "kata-item";
    if (item.type === "exercise") {
        const exerciseDiv = renderExercise(item as Exercise);
        itemDiv.append(exerciseDiv);
    }

    return itemDiv;
}

function renderKata(kata: Kata): HTMLDivElement {
    let kataDiv = document.createElement("div");

    // Render the content.
    let kataContent = document.createElement("div");
    kataContent.innerHTML = kata.contentAsHtml;
    kataDiv.append(kataContent);

    // Render each one of the items.
    for (let item of kata.items) {
        let renderedItem = renderItem(item);
        kataDiv.append(renderedItem);
    }
    return kataDiv;
}

export async function RenderKatas() {
    // Katas are rendered inside a div element with "katas-canvas" as id.
    let canvasDiv = document.querySelector('#katas-canvas') as HTMLDivElement;

    // Clear the katas' canvas every time before re-rendering.
    canvasDiv.innerHTML = "";

    // Render the selected kata.
    let katasDropdown = document.querySelector('#katas-list') as HTMLSelectElement;
    let selectedOption = katasDropdown.item(katasDropdown.selectedIndex)!;
    let kata = await getKata(selectedOption.value);
    let renderedKata = renderKata(kata);
    canvasDiv.append(renderedKata);

    // Render math stuff.
    MathJax.typeset();
}

export async function PopulateKatasList() {
    let katasDropdown = document.querySelector('#katas-list') as HTMLSelectElement;
    let katas = await getAllKatas();
    for (let kata of await getAllKatas()) {
        let option = document.createElement("option");
        option.value = kata.id;
        option.text = kata.title;
        katasDropdown.add(option);
    }
}