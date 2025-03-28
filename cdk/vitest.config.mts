import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        include: ["**/*.test.ts"],
        exclude: ["**/integ.*.test.ts", "node_modules/**/*.test.ts"],
    },
});
