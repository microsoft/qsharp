import { getCompilerWorker, getAllKatas, getKata, renderDump, QscEventTarget, type Kata, type KataItem, type Example, type Exercise, ShotResult } from "qsharp";

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

function renderShotResult(result: ShotResult): HTMLDivElement {
    let shotDiv = document.createElement("div");
    result.events.forEach(event => {
        switch (event.type) {
            case "Message":
                // A Message output
                let div = document.createElement("div");
                div.className = "message-output";
                div.innerText = event.message
                shotDiv.append(div);
                break;
            case "DumpMachine":
                // A DumpMachine output
                let table = document.createElement("table");
                table.innerHTML = renderDump(event.state);
                shotDiv.appendChild(table);
            default:
                let errorDiv = document.createElement("div");
                errorDiv.innerHTML = "Something else: " + event.type;
                shotDiv.append(errorDiv);
        }
    });

    const resultDiv = document.createElement("div");
    if (typeof result.result === 'string') {
        resultDiv.innerHTML = "Result: " + result.result;
    } else {
        const error = result.result;
        resultDiv.innerHTML = `${error.message}: [${error.start_pos}, ${error.end_pos}]`;
    }

    shotDiv.append(resultDiv);
    return shotDiv;
}

function renderExample(example: Example): HTMLDivElement {
    console.log("renderExample");
    let exampleDiv = document.createElement("div");
    let exampleContent = document.createElement("div");
    exampleContent.innerHTML = example.contentAsHtml;
    exampleDiv.append(exampleContent);
    let sourceCodeArea = document.createElement("textarea");
    sourceCodeArea.id = `source_${example.id}`;
    sourceCodeArea.rows = 30;
    sourceCodeArea.cols = 80;
    sourceCodeArea.value = example.source;
    exampleDiv.append(sourceCodeArea);
    let outputDiv = document.createElement("div");
    outputDiv.id = `ouput_${example.id}`;
    exampleDiv.append(outputDiv);
    let runButtonDiv = document.createElement("div");
    exampleDiv.append(runButtonDiv);
    let runButton = document.createElement("button");
    runButton.textContent = "Run";
    runButton.id = `run_${example.id}`;
    runButtonDiv.append(runButton);

    // Run the example when clicking the verify button.
    runButton.addEventListener('click', async _ => {
        outputDiv.innerHTML = "";
        let userCode = sourceCodeArea.value;
        try {
            const eventHandler = new QscEventTarget(true);
            const compiler = await getCompilerWorker("libs/worker.js");
            await compiler.run(userCode, "Kata.Main()", 1, eventHandler);
            let runResults = eventHandler.getResults();
            for (let result of runResults) {
                let renderedShotResult = renderShotResult(result);
                outputDiv.append(renderedShotResult);
            }
        } catch (e) {
            if (e instanceof Error) {
                outputDiv.innerHTML = "ERROR: " + e.message;
            }
        }
    });
    return exampleDiv;
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

    // Run the exercise when clicking the verify button.
    verifyButton.addEventListener('click', async _ => {
        outputDiv.innerHTML = "";
        let userCode = sourceCodeArea.value;
        try {
            const eventHandler = new QscEventTarget(true);
            const compiler = await getCompilerWorker("libs/worker.js");
            let _ = await compiler.runKata(userCode, exercise.verificationImplementation, eventHandler)
                .then(r => r)
                .catch(_ => false);
            for (let result of eventHandler.getResults()) {
                let renderedShotResult = renderShotResult(result);
                outputDiv.append(renderedShotResult);
            }
        } catch (e) {
            if (e instanceof Error) {
                outputDiv.innerHTML = "ERROR: " + e.message;
            }
        }
    });

    return exerciseDiv;
}

function renderItem(item: KataItem): HTMLDivElement {
    let itemDiv = document.createElement("div");
    itemDiv.className = "kata-item";
    if (item.type === "example") {
        const exampleDiv = renderExample(item as Example);
        itemDiv.append(exampleDiv);
    } else if (item.type === "exercise") {
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
    for (let kata of await getAllKatas()) {
        let option = document.createElement("option");
        option.value = kata.id;
        option.text = kata.title;
        katasDropdown.add(option);
    }
}