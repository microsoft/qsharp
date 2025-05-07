const fs = require("fs");
const path = require("path");

// Read package.json file
const packageJsonPath = path.join(process.cwd(), "package.json");
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
});

// Write the markdown content to a file
const outputPath = path.join(process.cwd(), "language-model-tools.md");
fs.writeFileSync(outputPath, markdownContent);

console.log(
  `Successfully extracted language model tools info to ${outputPath}`,
);
