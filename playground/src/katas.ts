import {getKataModule, queryKataModules, type KataExercise, type KataModule} from "qsharp/browser";


function renderModule(module: KataModule) : HTMLDivElement {
    let moduleDiv = document.createElement("div");
    moduleDiv.id = "kata-module";
    let moduleHeader = document.createElement("h2");
    moduleHeader.textContent = module.title;
    moduleDiv.append(moduleHeader);
    let moduleParagraph = document.createElement("p");
    moduleParagraph.textContent = module.description;
    moduleDiv.append(moduleParagraph);
    // TODO (cesarzc): Render exercise.
    return moduleDiv;
}

export function RenderKatas() {
    let canvasDiv = document.querySelector('#katas-canvas') as HTMLDivElement;
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