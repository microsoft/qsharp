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
      "**/.*/",
      "qdk_source/jupyterlab/lib/",
      "qdk_source/jupyterlab/qsharp-jupyterlab/labextension/",
      "qdk_source/npm/qsharp/dist/",
      "qdk_source/npm/qsharp/lib/",
      "qdk_source/npm/qsharp/src/*.generated.ts",
      "qdk_source/playground/public/",
      "qdk_source/vscode/out/",
      "qdk_source/vscode/test/out/",
      "qdk_source/widgets/src/qsharp_widgets/static/",
      "target/",
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
