# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# This script generates the code for the report page from the output_data.md file.
# To run, simply execute `python generate_report_code.py` in the npm/ux folder.
# copy the output and paste it into the report.tsx file.
# It provides a code for the CreateReport function.

import re

parse = False

title = ""
always_visible = True
first_entry = True
entries = []

label = ""
path = ""
value = ""
description = ""
explanation = ""

assumptions = []

ignore_paths = [
    "floquet_code",
    "surface_code",
    "qubit_gate_ns_e3",
    "qubit_gate_ns_e4",
    "qubit_gate_us_e3",
    "qubit_gate_us_e4",
    "qubit_maj_ns_e4",
    "qubit_maj_ns_e6",
]

path_map = {
    "errorBudget/rotations": "result.errorBudget.rotations",
    "jobParams/qecScheme/crossingPrefactor": "result.jobParams.qecScheme.crossingPrefactor",
    "jobParams/qecScheme/errorCorrectionThreshold": "result.jobParams.qecScheme.errorCorrectionThreshold",
    "jobParams/qecScheme/logicalCycleTime": "result.jobParams.qecScheme.logicalCycleTime",
    "jobParams/qecScheme/physicalQubitsPerLogicalQubit": "result.jobParams.qecScheme.physicalQubitsPerLogicalQubit",
    "jobParams/qubitParams/tGateErrorRate": "result.jobParams.qubitParams.tGateErrorRate",
    "logicalCounts/ccixCount": "numberFormat.format(result.logicalCounts.ccixCount)",
    "logicalCounts/cczCount": "numberFormat.format(result.logicalCounts.cczCount)",
    "logicalCounts/measurementCount": "numberFormat.format(result.logicalCounts.measurementCount)",
    "logicalCounts/numQubits": "numberFormat.format(result.logicalCounts.numQubits)",
    "logicalCounts/rotationCount": "numberFormat.format(result.logicalCounts.rotationCount)",
    "logicalCounts/rotationDepth": "numberFormat.format(result.logicalCounts.rotationDepth)",
    "logicalCounts/tCount": "numberFormat.format(result.logicalCounts.tCount)",
    "logicalQubit/codeDistance": "result.logicalQubit.codeDistance",
    "logicalQubit/logicalCyclesPerSecond": "numberFormatF64.format(result.physicalCounts.breakdown.clockFrequency)",
    "logicalQubit/logicalCycleTime": "numberFormat.format(result.logicalQubit.logicalCycleTime)",
    "logicalQubit/physicalQubits": "numberFormat.format(result.logicalQubit.physicalQubits)",
    "physicalCounts/breakdown/algorithmicLogicalDepth": "numberFormat.format(result.physicalCounts.breakdown.algorithmicLogicalDepth)",
    "physicalCounts/breakdown/algorithmicLogicalQubits": "numberFormat.format(result.physicalCounts.breakdown.algorithmicLogicalQubits)",
    "physicalCounts/breakdown/cliffordErrorRate": "result.physicalCounts.breakdown.cliffordErrorRate",
    "physicalCounts/breakdown/logicalDepth": "numberFormat.format(result.physicalCounts.breakdown.logicalDepth)",
    "physicalCounts/breakdown/physicalQubitsForAlgorithm": "numberFormat.format(result.physicalCounts.breakdown.physicalQubitsForAlgorithm)",
    "physicalCounts/breakdown/physicalQubitsForTfactories": "numberFormat.format(result.physicalCounts.breakdown.physicalQubitsForTfactories)",
    "physicalCounts/breakdown/numTfactories": "numberFormat.format(result.physicalCounts.breakdown.numTfactories)",
    "physicalCounts/breakdown/numTfactoryRuns": "numberFormat.format(result.physicalCounts.breakdown.numTfactoryRuns)",
    "physicalCounts/breakdown/numTstates": "numberFormat.format(result.physicalCounts.breakdown.numTstates)",
    "physicalCounts/breakdown/requiredLogicalQubitErrorRate": "result.physicalCounts.breakdown.requiredLogicalQubitErrorRate",
    "physicalCounts/physicalQubits": "numberFormat.format(result.physicalCounts.physicalQubits)",
    "physicalCounts/runtime": "numberFormat.format(result.physicalCounts.runtime)",
    "physicalCountsFormatted/clockFrequency": "result.physicalCountsFormatted.clockFrequency",
    "physicalCountsFormatted/errorBudget": "result.physicalCountsFormatted.errorBudget",
    "physicalCountsFormatted/errorBudgetLogical": "result.physicalCountsFormatted.errorBudgetLogical",
    "physicalCountsFormatted/errorBudgetRotations": "result.physicalCountsFormatted.errorBudgetRotations",
    "physicalCountsFormatted/errorBudgetTstates": "result.physicalCountsFormatted.errorBudgetTstates",
    "physicalCountsFormatted/logicalCycleTime": "result.physicalCountsFormatted.logicalCycleTime",
    "physicalCountsFormatted/numTsPerRotation": "result.physicalCountsFormatted.numTsPerRotation",
    "physicalCountsFormatted/requiredLogicalQubitErrorRate": "result.physicalCountsFormatted.requiredLogicalQubitErrorRate",
    "physicalCountsFormatted/requiredLogicalTstateErrorRate": "result.physicalCountsFormatted.requiredLogicalTstateErrorRate",
    "physicalCountsFormatted/runtime": "result.physicalCountsFormatted.runtime",
    "physicalCountsFormatted/tfactoryRuntime": "result.physicalCountsFormatted.tfactoryRuntime",
    "physicalCountsFormatted/tstateLogicalErrorRate": "result.physicalCountsFormatted.tstateLogicalErrorRate",
    "tfactory/physicalQubits": "numberFormat.format(result.tfactory == null ? 0 : result.tfactory.physicalQubits)",
    "tfactory/numInputTstates": "numberFormat.format(result.tfactory == null ? 0 : result.tfactory.numInputTstates)",
    "tfactory/numTstates": "numberFormat.format(result.tfactory == null ? 0 : result.tfactory.numTstates)",
    "tfactory/runtime": "numberFormat.format(result.tfactory == null ? 0 : result.tfactory.runtime)",
}

print()
print()
print(
    "// THIS CODE HAS BEEN AUTOMATICALLY GENERATED WITH generate_report_code.py from output_data.md"
)
print("export function CreateReport(result: ReData): ReportData {")
print("    const groups = [] as ReportGroup[];")
print("    let entries = [] as ReportEntry[];")
print("    const numberFormat = new Intl.NumberFormat();")
print(
    "    const numberFormatF64 = new Intl.NumberFormat(undefined, { maximumFractionDigits: 2, minimumFractionDigits: 2,});"
)
print()


def add_group():
    global always_visible, entries, title

    if len(entries) != 0:
        if title == "T factory parameters":
            print("    if (result.tfactory != null) {")

        always_visible_str = "true" if always_visible else "false"
        print("    entries = [];")
        for path, label, description, explanation in entries:
            if path in [
                "jobParams/qubitParams/oneQubitGateTime",
                "jobParams/qubitParams/twoQubitGateTime",
                "jobParams/qubitParams/oneQubitGateErrorRate",
                "jobParams/qubitParams/twoQubitGateErrorRate",
            ]:
                print(
                    '    if (result.jobParams.qubitParams.instructionSet == "GateBased") {'
                )
                print(
                    f'        entries.push({{path: "{path}", label: "{label}", description: {description}, explanation: {explanation}}});'
                )
                print("        }")
            elif path in [
                "jobParams/qubitParams/twoQubitJointMeasurementTime",
                "jobParams/qubitParams/twoQubitJointMeasurementErrorRate",
            ]:
                print(
                    '    if (result.jobParams.qubitParams.instructionSet == "Majorana") {'
                )
                print(
                    f'        entries.push({{path: "{path}", label: "{label}", description: {description}, explanation: {explanation}}});'
                )
                print("    }")
            else:
                print(
                    f'        entries.push({{path: "{path}", label: "{label}", description: {description}, explanation: {explanation}}});'
                )
        print(
            f'    groups.push({{ title: "{title}", alwaysVisible: {always_visible_str}, entries: entries }});'
        )
        print()

        if title == "T factory parameters":
            print("    }")

        always_visible = False
        entries.clear()


def create_fmt_string(string):
    args = []

    # Find in-text `paths`
    pos = string.find("`")
    while pos != -1:
        pos2 = string.find("`", pos + 1)
        path = string[pos + 1 : pos2]

        if path in ignore_paths:
            pos = string.find("`", pos2 + 1)
        else:
            string = string[:pos] + "${" + path_map[path] + "}" + string[pos2 + 1 :]
            pos = string.find("`", pos + 1)

    # Find in-math \mathtt{paths}
    pos = string.find("\\mathtt{")
    while pos != -1:
        pos2 = string.find("}", pos + 1)
        path = string[pos + 8 : pos2]

        string = string[:pos] + "${" + path_map[path] + "}" + string[pos2 + 1 :]
        pos = string.find("\\mathtt{", pos + 1)

    if len(args) != 0:
        args_list = ", ".join(args)

        cur = 0
        while cur < len(string):
            if string[cur] == "{" and string[cur + 1] != "}":
                string = string[:cur] + "{" + string[cur:]
                cur += 2
            elif string[cur] == "}" and string[cur - 1] != "{":
                string = string[:cur] + "}" + string[cur:]
                cur += 2
            else:
                cur += 1

    string = string.replace("\\", "\\\\")
    string = string.replace("`", "\\`")
    return f"`{string}`"


with open("output_data.md", "r") as f:
    for line in f.readlines():
        line = line.strip()

        if line == "":
            continue

        if not parse:
            if line.startswith("## "):
                parse = True
            else:
                continue

        if line.startswith("## "):
            # Finish previous group?
            if len(entries) != 0:
                add_group()

            title = line[3:].strip()
        elif line.startswith("### "):
            label = line[4:].strip()
        elif line.startswith("[//]: #"):
            path = line[9:-1]
        elif line.startswith("_"):
            description = line[1:-1]
        elif line.startswith("-"):
            assumptions.append(line[2:])
        else:
            explanation = line

            entries.append(
                (
                    path,
                    label,
                    create_fmt_string(description),
                    create_fmt_string(explanation),
                )
            )

# Add assumptions
assert title == "Assumptions"
print("    const assumptions = [")
for assumption in assumptions:
    print(f"        '{assumption}',")
print("    ];")
print()
print("    return { groups: groups, assumptions: assumptions };")
print("}")
print("// END OF AUTOMATICALLY GENERATED CODE")
print()
print()
