import { getCompilerWorker, getAllKatas, getKata, renderDump, QscEventTarget, type Kata, type KataItem, type Example, type Exercise, ShotResult } from "qsharp";

// MathJax will already be loaded on the page. Need to call `typeset` when LaTeX content changes.
declare var MathJax: { typeset: () => void; };

function renderShotResult(result: ShotResult): HTMLDivElement {
    const shotDiv = document.createElement("div");
    result.events.forEach(event => {
        switch (event.type) {
            case "Message":
                // A Message output
                const div = document.createElement("div");
                div.className = "message-output";
                div.innerText = event.message
                shotDiv.append(div);
                break;
            case "DumpMachine":
                // A DumpMachine output
                const table = document.createElement("table");
                table.innerHTML = renderDump(event.state);
                shotDiv.appendChild(table);
                break;
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
    const exampleDiv = document.createElement("div");
    const exampleContent = document.createElement("div");
    exampleContent.innerHTML = example.contentAsHtml;
    exampleDiv.append(exampleContent);
    const sourceCodeArea = document.createElement("textarea");
    sourceCodeArea.id = `source_${example.id}`;
    sourceCodeArea.rows = 30;
    sourceCodeArea.cols = 80;
    sourceCodeArea.value = example.source;
    exampleDiv.append(sourceCodeArea);
    const outputDiv = document.createElement("div");
    outputDiv.id = `ouput_${example.id}`;
    exampleDiv.append(outputDiv);
    const runButtonDiv = document.createElement("div");
    exampleDiv.append(runButtonDiv);
    const runButton = document.createElement("button");
    runButton.textContent = "Run";
    runButton.id = `run_${example.id}`;
    runButtonDiv.append(runButton);

    // Run the example when clicking the verify button.
    runButton.addEventListener('click', async _ => {
        outputDiv.innerHTML = "";
        const userCode = sourceCodeArea.value;
        try {
            const eventHandler = new QscEventTarget(true);
            const compiler = await getCompilerWorker("libs/worker.js");
            await compiler.run(userCode, "Kata.Main()", 1, eventHandler);
            const runResults = eventHandler.getResults();
            for (const result of runResults) {
                const renderedShotResult = renderShotResult(result);
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
    const exerciseDiv = document.createElement("div");
    const exerciseContent = document.createElement("div");
    exerciseContent.innerHTML = exercise.contentAsHtml;
    exerciseDiv.append(exerciseContent);
    const sourceCodeArea = document.createElement("textarea");
    sourceCodeArea.id = `source_${exercise.id}`;
    sourceCodeArea.rows = 30;
    sourceCodeArea.cols = 80;
    sourceCodeArea.value = exercise.placeholderImplementation;
    exerciseDiv.append(sourceCodeArea);
    const outputDiv = document.createElement("div");
    outputDiv.id = `ouput_${exercise.id}`;
    exerciseDiv.append(outputDiv);
    const verifyButtonDiv = document.createElement("div");
    exerciseDiv.append(verifyButtonDiv);
    const verifyButton = document.createElement("button");
    verifyButton.textContent = "Verify";
    verifyButton.id = `verify_${exercise.id}`;
    verifyButtonDiv.append(verifyButton);

    // Run the exercise when clicking the verify button.
    verifyButton.addEventListener('click', async _ => {
        outputDiv.innerHTML = "";
        const userCode = sourceCodeArea.value;
        try {
            const eventHandler = new QscEventTarget(true);
            const compiler = await getCompilerWorker("libs/worker.js");
            const _ = await compiler.runKata(userCode, exercise.verificationImplementation, eventHandler);
            for (const result of eventHandler.getResults()) {
                const renderedShotResult = renderShotResult(result);
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
    const itemDiv = document.createElement("div");
    itemDiv.className = "kata-item";
    if (item.type === "example") {
        const exampleDiv = renderExample(item);
        itemDiv.append(exampleDiv);
    } else if (item.type === "exercise") {
        const exerciseDiv = renderExercise(item);
        itemDiv.append(exerciseDiv);
    }

    return itemDiv;
}

function renderKata(kata: Kata): HTMLDivElement {
    const kataDiv = document.createElement("div");

    // Render the content.
    const kataContent = document.createElement("div");
    kataContent.innerHTML = kata.contentAsHtml;
    kataDiv.append(kataContent);

    // Render each one of the items.
    for (const item of kata.items) {
        const renderedItem = renderItem(item);
        kataDiv.append(renderedItem);
    }
    return kataDiv;
}

async function renderKatas() {
    // Katas are rendered inside a div element with "katas-canvas" as id.
    const canvasDiv = document.querySelector('#katas-canvas') as HTMLDivElement;

    // Clear the katas' canvas every time before re-rendering.
    canvasDiv.innerHTML = "";

    // Render the selected kata.
    const katasDropdown = document.querySelector('#katas-list') as HTMLSelectElement;
    const selectedOption = katasDropdown.item(katasDropdown.selectedIndex)!;
    const kata = await getKata(selectedOption.value);
    const renderedKata = renderKata(kata);
    canvasDiv.append(renderedKata);

    // Render math stuff.
    MathJax.typeset();
}

async function populateKatasList() {
    const katasDropdown = document.querySelector('#katas-list') as HTMLSelectElement;
    for (const kata of await getAllKatas()) {
        const option = document.createElement("option");
        option.value = kata.id;
        option.text = kata.title;
        katasDropdown.add(option);
    }
}

export async function ShowKatas() {
    let katasDiv = document.querySelector('#katas') as HTMLDivElement;
    let katasSelect = document.createElement("select") as HTMLSelectElement;
    katasSelect.id = "katas-list";
    katasDiv.prepend(katasSelect);
    katasDiv.prepend();
    let katasTitle = document.createElement("h1");
    katasTitle.textContent = "Katas";
    katasDiv.prepend(katasTitle);
    populateKatasList()
        .then(() => renderKatas())
        .then(() => {
            katasSelect.addEventListener('change', _ => {
                renderKatas();
            });
        });
}