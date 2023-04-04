import {getKataModule, queryKataModules, type KataExercise, type KataModule} from "qsharp/browser";

function renderExercise(exercise: KataExercise) : HTMLDivElement {
    let exerciseDiv = document.createElement("div");
    exerciseDiv.id = "kata-exercise";
    let exerciseHeader = document.createElement("h3");
    exerciseHeader.textContent = exercise.title;
    exerciseDiv.append(exerciseHeader);
    let exerciseParagraph = document.createElement("p");
    exerciseParagraph.textContent = exercise.description;
    exerciseDiv.append(exerciseParagraph);
    let sourceCodeArea = document.createElement("textarea");
    sourceCodeArea.textContent = exercise.placeholderImplementation;
    exerciseDiv.append(sourceCodeArea);
    let verifyButton = document.createElement("button");
    verifyButton.textContent = "Verify";
    verifyButton.id = `verify-${exercise.id}`;
    let verifyDiv = document.createElement("div");
    verifyDiv.append(verifyButton);
    exerciseDiv.append(verifyDiv);
    return exerciseDiv;
}

function renderModule(module: KataModule) : HTMLDivElement {
    let moduleDiv = document.createElement("div");
    moduleDiv.id = "kata-module";

    // Render the title and the description.
    let moduleHeader = document.createElement("h2");
    moduleHeader.textContent = module.title;
    moduleDiv.append(moduleHeader);
    let moduleParagraph = document.createElement("p");
    moduleParagraph.textContent = module.description;
    moduleDiv.append(moduleParagraph);

    // Render each one of the module exercises.
    for (let exercise of module.exercises)
    {
        let renderedExercise = renderExercise(exercise);
        moduleDiv.append(renderedExercise);
    }
    return moduleDiv;
}

export function RenderKatas() {
    // Katas are rendered inside a div element with "katas-canvas" as id.
    let canvasDiv = document.querySelector('#katas-canvas') as HTMLDivElement;

    // Clear the katas' canvas every time before re-rendering.
    while (canvasDiv.hasChildNodes()) {
        canvasDiv.removeChild(canvasDiv.lastChild!);
    }

    // Render the selected module.
    let modulesDropdown = document.querySelector('#modules') as HTMLSelectElement;
    let selectedOption = modulesDropdown.item(modulesDropdown.selectedIndex)!;
    let module = getKataModule(selectedOption.value);
    let renderedModule = renderModule(module);
    canvasDiv.append(renderedModule);
}

export function PopulateModules() {
    let modulesDropdown = document.querySelector('#modules') as HTMLSelectElement;
    for (let module of queryKataModules())
    {
        let option = document.createElement("option");
        option.value = module.id;
        option.text = module.title;
        modulesDropdown.add(option);
    }
}