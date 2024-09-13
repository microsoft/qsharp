import globals from "globals";
import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  {
    ignores: [
      "target/",
      "playground/public/",
      "npm/qsharp/dist/",
      "npm/qsharp/lib/",
      "npm/qsharp/src/*.generated.ts",
      "jupyterlab/lib/",
      "jupyterlab/qsharp-jupyterlab/labextension/",
      "**/.*/",
      "vscode/out/",
      "vscode/test/out/",
      "widgets/src/qsharp_widgets/static/",
    ],
  },
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
);
