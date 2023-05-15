/* eslint-env node */
module.exports = {
  extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended"],
  parser: "@typescript-eslint/parser",
  plugins: ["@typescript-eslint"],
  root: true,
  ignorePatterns: [
    "target/",
    "playground/public/",
    "npm/dist/",
    "npm/lib/",
    "vscode/out/",
    "jupyterlab/lib",
    "jupyterlab/qsharp_jupyterlab/labextension"
  ],
  env: {
    browser: true,
    node: true,
  },
};
