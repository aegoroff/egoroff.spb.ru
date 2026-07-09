"use strict";

const { run } = require("vue-tsc");

// vue-tsc still embeds TypeScript 6 until TS7 exposes a stable programmatic API.
run(require.resolve("@typescript/old/lib/tsc"));
