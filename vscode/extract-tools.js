const fs = require("fs");
const path = require("path");

const name = process.argv[2] || "package.json";
if (!fs.existsSync(name)) {
  console.error(`File not found: ${name}`);
  process.exit(1);
}

// Read package.json file
const packageJsonPath = path.join(process.cwd(), name);
const packageJson = require(packageJsonPath);

// Extract the languageModelTools array
const tools = packageJson.contributes.languageModelTools;

// Prepare the markdown content
let markdownContent = "# Q# Language Model Tools\n\n";

// Loop through each tool and extract the required fields
tools.forEach((tool) => {
  markdownContent += `## ${tool.displayName}\n\n`;
  markdownContent += `- **Name**: \`${tool.name}\`\n`;
  markdownContent += `- **Description**: ${tool.modelDescription}\n\n`;

  // Extract and format input parameters if available
  if (tool.inputSchema && tool.inputSchema.properties) {
    markdownContent += "### Input Parameters\n\n";

    // Create a parameters table with name, type, required, and description
    markdownContent += "| Parameter | Type | Required | Description |\n";
    markdownContent += "|-----------|------|----------|-------------|\n";

    const properties = tool.inputSchema.properties;
    const requiredParams = tool.inputSchema.required || [];

    Object.keys(properties).forEach((paramName) => {
      const param = properties[paramName];
      const isRequired = requiredParams.includes(paramName) ? "Yes" : "No";
      markdownContent += `| \`${paramName}\` | \`${param.type}\` | ${isRequired} | ${param.description} |\n`;
    });

    markdownContent += "\n";
  }
});

// Write the markdown content to a file
const outputPath = path.join(process.cwd(), `language-model-tools-${name}.md`);
fs.writeFileSync(outputPath, markdownContent);

console.log(
  `Successfully extracted language model tools info to ${outputPath}`,
);
