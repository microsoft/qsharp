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
    "logicalCounts/ccixCount": "formatThousandSep(result.logicalCounts.fccixCount)",
    "logicalCounts/cczCount": "formatThousandSep(result.logicalCounts.cczCount)",
    "logicalCounts/measurementCount": "formatThousandSep(result.logicalCounts.measurementCount)",
    "logicalCounts/numQubits": "formatThousandSep(result.logicalCounts.numQubits)",
    "logicalCounts/rotationCount": "formatThousandSep(result.logicalCounts.rotationCount)",
    "logicalCounts/rotationDepth": "formatThousandSep(result.logicalCounts.rotationDepth)",
    "logicalCounts/tCount": "formatThousandSep(result.logicalCounts.tCount)",
    "logicalQubit/codeDistance": "result.logicalQubit.codeDistance",
    "logicalQubit/logicalCyclesPerSecond": "formatThousandSepF64(result.logicalQubit.logicalCyclesPerSecond)",
    "logicalQubit/logicalCycleTime": "formatThousandSep(result.logicalQubit.logicalCycleTime)",
    "logicalQubit/physicalQubits": "formatThousandSep(result.logicalQubit.physicalQubits)",
    "physicalCounts/breakdown/algorithmicLogicalDepth": "formatThousandSep(result.algorithmicLogicalDepth)",
    "physicalCounts/breakdown/algorithmicLogicalQubits": "formatThousandSep(result.layoutOverhead.logicalQubits)",
    "physicalCounts/breakdown/cliffordErrorRate": "result.logicalQubit.physicalQubit.cliffordErrorRate",
    "physicalCounts/breakdown/logicalDepth": "formatThousandSep(result.physcialCounts.breakdown.numCycles)",
    "physicalCounts/breakdown/physicalQubitsForAlgorithm": "formatThousandSep(result.physcialCounts.breakdown.physicalQubitsForAlgorithm)",
    "physicalCounts/breakdown/physicalQubitsForTfactories": "formatThousandSep(result.physcialCounts.breakdown.physicalQubitsForTfactories)",
    "physicalCounts/breakdown/numTfactories": "formatThousandSep(result.physcialCounts.breakdown.numTfactories)",
    "physicalCounts/breakdown/numTfactoryRuns": "formatThousandSep(result.physcialCounts.breakdown.numTfactoryRuns)",
    "physicalCounts/breakdown/numTstates": "formatThousandSep(result.physcialCounts.breakdown.numTstates)",
    "physicalCounts/breakdown/requiredLogicalQubitErrorRate": "result.physcialCounts.breakdown.requiredLogicalQubitErrorRate",
    "physicalCounts/physicalQubits": "formatThousandSep(result.physcialCounts.breakdown.physicalQubits)",
    "physicalCounts/runtime": "formatThousandSep(result.physcialCounts.breakdown.runtime)",
    "physicalCountsFormatted/clockFrequency": "result.physcialCountFormatted.clockFrequency",
    "physicalCountsFormatted/errorBudget": "result.physcialCountFormatted.errorBudget",
    "physicalCountsFormatted/errorBudgetLogical": "result.physcialCountFormatted.errorBudgetLogical",
    "physicalCountsFormatted/errorBudgetRotations": "result.physcialCountFormatted.errorBudgetRotations",
    "physicalCountsFormatted/errorBudgetTstates": "result.physcialCountFormatted.errorBudgetTstates",
    "physicalCountsFormatted/logicalCycleTime": "result.physcialCountFormatted.logicalCycleTime",
    "physicalCountsFormatted/numTsPerRotation": "result.physcialCountFormatted.numTsPerRotation",
    "physicalCountsFormatted/requiredLogicalQubitErrorRate": "result.physcialCountFormatteds.requiredLogicalQubitErrorRate",
    "physicalCountsFormatted/requiredLogicalTstateErrorRate": "result.physcialCountFormatted.requiredLogicalTstateErrorRate",
    "physicalCountsFormatted/runtime": "result.physcialCountFormatted.runtime",
    "physicalCountsFormatted/tfactoryRuntime": "result.physcialCountFormatted.tfactory_runtime",
    "physicalCountsFormatted/tstateLogicalErrorRate": "result.physcialCountFormatted.tstateLogicalErrorRate",
    "tfactory/physicalQubits": "formatThousandSep(result.tfactory == null ? 0 result.tfactory.physical_qubits)",
    "tfactory/numInputTstates": "formatThousandSep(result.tfactory == null ? 0 result.tfactory.input_t_count)",
    "tfactory/numTstates": "formatThousandSep(result.tfactory == null ? 0 result.tfactory.output_t_count)",
    "tfactory/runtime": "formatThousandSep(result.tfactory == null ? 0 result.tfactory.duration)",
}

print(
    "        // THIS CODE HAS BEEN AUTOMATICALLY GENERATED WITH generate_report_code.py from output_data.md"
)
print("        const groups = [] as ReportGroup[];")
print("        let entries = [] as ReportEntry[];")
print()


def add_group():
    global always_visible, entries, title

    if len(entries) != 0:
        if title == "T factory parameters":
            print("        if (result.tfactory != null) {")

        always_visible_str = "true" if always_visible else "false"
        print("        entries = [];")
        for path, label, description, explanation in entries:
            if path in [
                "jobParams/qubitParams/oneQubitGateTime",
                "jobParams/qubitParams/twoQubitGateTime",
                "jobParams/qubitParams/oneQubitGateErrorRate",
                "jobParams/qubitParams/twoQubitGateErrorRate",
            ]:
                print(
                    '        if (result.jobParams.qubitParams.instructionSet == "GateBased") {'
                )
                print(
                    f'            entries.push({{path: "{path}", label: "{label}", description: {description}, explanation: {explanation}}});'
                )
                print("        }")
            elif path in [
                "jobParams/qubitParams/twoQubitJointMeasurementTime",
                "jobParams/qubitParams/twoQubitJointMeasurementErrorRate",
            ]:
                print(
                    '        if (result.jobParams.qubitParams.instructionSet == "Majorana") {'
                )
                print(
                    f'            entries.push({{path: "{path}", label: "{label}", description: {description}, explanation: {explanation}}});'
                )
                print("        }")
            else:
                print(
                    f'            entries.push({{path: "{path}", label: "{label}", description: {description}, explanation: {explanation}}});'
                )
        print(
            f'        groups.push({{ title: "{title}", alwaysVisible: {always_visible_str}, entries: entries }});'
        )
        print()

        if title == "T factory parameters":
            print("        }")

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
            string = string[:pos] + "{" + path_map[path] + "}" + string[pos2 + 1 :]
            pos = string.find("`", pos + 1)

    # Find in-math \mathtt{paths}
    pos = string.find("\\mathtt{")
    while pos != -1:
        pos2 = string.find("}", pos + 1)
        path = string[pos + 8 : pos2]

        string = string[:pos] + "{" + path_map[path] + "}" + string[pos2 + 1 :]
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

        # return f'&format!(r#"{string}"#, {args_list})'
        string = string.replace("`", "\\`")
        string = string.replace("'", "\\'")
        return f"'{string}'"
    else:
        string = string.replace("`", "\\`")
        string = string.replace("'", "\\'")
        return f"'{string}'"


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
print("        const assumptions = [")
for assumption in assumptions:
    print(f"            '{assumption}',")
print("        ];")
