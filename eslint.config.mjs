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
      "qdk_source/frontend/jupyterlab/lib/",
      "qdk_source/frontend/jupyterlab/qsharp-jupyterlab/labextension/",
      "qdk_source/frontend/npm/qsharp/dist/",
      "qdk_source/frontend/npm/qsharp/lib/",
      "qdk_source/frontend/npm/qsharp/src/*.generated.ts",
      "qdk_source/frontend/playground/public/",
      "qdk_source/frontend/vscode/out/",
      "qdk_source/frontend/vscode/test/out/",
      "qdk_source/frontend/widgets/src/qsharp_widgets/static/",
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
