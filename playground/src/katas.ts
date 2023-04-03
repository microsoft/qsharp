
export function RenderKatas() {
    alert("RenderKata");
}

export function PopulateModules() {
    let modulesDropdown = document.querySelector('#modules') as HTMLSelectElement;
    let option = document.createElement("option");
    option.value = "sample-module";
    option.text = "Sample Module"
    modulesDropdown.add(option);
}