import * as fs from "fs";
import * as esbuild from "esbuild";
import * as path from "path";

const PATHS_TO_HANDLER = fs
    .readdirSync(__dirname, { withFileTypes: true, recursive: true })
    .filter((element) => element.isDirectory())
    .map((element) => path.join(element.parentPath, element.name, "handler.ts"))
    .filter((file) => fs.existsSync(file));

const OUTDIR = path.join(__dirname, "..", "..", "build", "function");

(async () => {
    await esbuild.build({
        entryPoints: PATHS_TO_HANDLER,
        bundle: true,
        minify: true,
        platform: "node",
        target: "es2022",
        external: ["@aws-sdk"],
        outdir: OUTDIR,
        outbase: __dirname,
    });

    fs.readdirSync(OUTDIR, { withFileTypes: true, recursive: true })
        .filter((element) => element.isDirectory())
        .filter((element) => fs.existsSync(path.join(element.parentPath, element.name, "handler.js")))
        .forEach((element) =>
            fs.renameSync(
                path.join(element.parentPath, element.name, "handler.js"),
                path.join(element.parentPath, element.name, "index.js"),
            ),
        );
})();
