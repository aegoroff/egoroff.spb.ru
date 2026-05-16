import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import pluginVue from "eslint-plugin-vue";

export default [
  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts,vue}"],
    ...js.configs.recommended,
    languageOptions: {
      globals: globals.browser
    }
  },
  ...tseslint.configs.recommended,
  ...pluginVue.configs["flat/essential"],
  {
    files: ["**/*.vue"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser
      }
    }
  },
  {
    ignores: ["**/dist/**", "**/node_modules/**", "**/*.d.ts"]
  }
];