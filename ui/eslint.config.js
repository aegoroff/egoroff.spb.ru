const js = require("@eslint/js");
const globals = require("globals");
const tseslint = require("typescript-eslint");
const pluginVue = require("eslint-plugin-vue");

module.exports = [
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
